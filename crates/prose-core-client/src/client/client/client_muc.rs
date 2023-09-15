// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{bail, Result};
use jid::{BareJid, Jid};
use prose_xmpp::stanza::muc::{mediated_invite, DirectInvite, MediatedInvite};
use prose_xmpp::stanza::ConferenceBookmark;
use prose_xmpp::{mods, RequestError};
use std::iter;
use tracing::info;
use xmpp_parsers::bookmarks2::{Autojoin, Conference};
use xmpp_parsers::muc::user::Affiliation;

use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::types::muc::{BookmarkMetadata, Room, RoomMetadata, RoomSettings};
use crate::types::{muc, Bookmarks};
use crate::util::StringExt;

use super::Client;

#[derive(thiserror::Error, Debug, PartialEq)]
enum MUCError {
    #[error("Server does not support MUC (XEP-0045)")]
    Unsupported,
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub fn connected_rooms(&self) -> Vec<Room> {
        self.inner
            .connected_rooms
            .read()
            .values()
            .cloned()
            .collect()
    }

    pub async fn load_public_rooms(&self) -> Result<Vec<mods::muc::Room>> {
        self.muc_service()?.load_public_rooms().await
    }

    pub async fn create_group(&self, participants: &[BareJid]) -> Result<()> {
        if participants.is_empty() {
            bail!("Group must have at least one other participant.")
        }

        // Load participant infos so that we can build a nice human-readable name for the group…
        let our_jid = self.connected_jid()?.to_bare();
        let mut participant_names = vec![];

        for jid in participants.iter().chain(iter::once(&our_jid)) {
            let participant_name = self
                .load_user_profile(jid.clone(), Default::default())
                .await?
                .and_then(|profile| profile.first_name.or(profile.nickname))
                .or(jid.node_str().map(|node| node.to_uppercase_first_letter()))
                .unwrap_or(jid.to_string());
            participant_names.push(participant_name);
        }
        participant_names.sort();

        let group_name = participant_names.join(", ");

        let metadata = self
            .muc_service()?
            .create_or_join_group(&group_name, participants)
            .await?;

        let room_has_been_created = metadata.room_has_been_created();
        let room = Room::from(metadata);
        let room_jid = room.jid().clone();

        // So we were actually already connected to that room.
        if self.inner.connected_rooms.read().contains_key(&room.jid()) {
            return Ok(());
        }

        // Try to promote all participants to owners…
        info!("Update participant affiliations…");
        let muc_mod = self.client.get_mod::<mods::MUC>();
        muc_mod
            .update_user_affiliations(
                &room_jid,
                participants
                    .iter()
                    .map(|jid| (jid.clone(), Affiliation::Owner)),
            )
            .await?;

        let bookmark = ConferenceBookmark {
            jid: room.jid().clone().into(),
            conference: Conference {
                autojoin: Autojoin::True,
                name: Some(group_name),
                nick: Some(room.nick().to_string()),
                password: None,
                extensions: vec![BookmarkMetadata {
                    room_type: muc::RoomType::Group,
                    participants: Some(participants.iter().map(Clone::clone).collect()),
                }
                .into()],
            },
        };

        // Add room to our connected rooms and notify delegate…
        self.did_enter_room(room);

        // Save bookmark for created (or joined) room…
        self.insert_and_publish_bookmark(bookmark).await?;

        // If the room existed already we won't send any invites again.
        if !room_has_been_created {
            return Ok(());
        }

        // Send invites…
        info!("Sending invites for created group…");
        muc_mod
            .send_mediated_invite(
                &room_jid,
                MediatedInvite {
                    invites: participants
                        .into_iter()
                        .map(|participant| mediated_invite::Invite {
                            from: None,
                            to: Some(participant.clone().into()),
                            reason: None,
                        })
                        .collect(),
                    password: None,
                },
            )
            .await?;

        Ok(())
    }

    pub async fn create_public_channel(&self, channel_name: impl AsRef<str>) -> Result<()> {
        // Create room…
        info!(
            "Creating public channel with name {}…",
            channel_name.as_ref()
        );

        let room: Room = self
            .muc_service()?
            .create_or_join_public_channel(channel_name.as_ref())
            .await?
            .into();

        let bookmark = ConferenceBookmark {
            jid: room.jid().clone().into(),
            conference: Conference {
                autojoin: Autojoin::True,
                name: Some(channel_name.as_ref().to_string()),
                nick: Some(room.nick().to_string()),
                password: None,
                extensions: vec![BookmarkMetadata {
                    room_type: muc::RoomType::PublicChannel,
                    participants: None,
                }
                .into()],
            },
        };

        // Add room to our connected rooms and notify delegate…
        self.did_enter_room(room);

        // Save bookmark for created (or joined) channel…
        self.insert_and_publish_bookmark(bookmark).await?;

        Ok(())
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub(super) async fn load_and_connect_bookmarks(&self) -> Result<()> {
        let bookmark_mod = self.client.get_mod::<mods::Bookmark>();
        let bookmark2_mod = self.client.get_mod::<mods::Bookmark2>();

        let bookmarks = Bookmarks::new(
            bookmark_mod.load_bookmarks().await?,
            bookmark2_mod.load_bookmarks().await?,
        );

        for bookmark in bookmarks.iter() {
            self.enter_room_if_needed(
                &bookmark.jid.to_bare(),
                bookmark.conference.nick.as_deref(),
                bookmark.conference.password.as_deref(),
            )
            .await?;
        }

        *self.inner.bookmarks.write() = bookmarks;

        Ok(())
    }

    pub(super) async fn handle_direct_invite(&self, from: Jid, invite: DirectInvite) -> Result<()> {
        Ok(())
    }

    pub(super) async fn handle_mediated_invite(
        &self,
        from: Jid,
        invite: MediatedInvite,
    ) -> Result<()> {
        println!("DID RECEIVE MEDIATED INVITE FROM {}: {:?}", from, invite);

        // // TODO: Handle this properly
        //
        // self.insert_and_publish_bookmark(ConferenceBookmark {
        //     jid: from,
        //     conference: Conference {
        //         autojoin: Autojoin::True,
        //         name: None,
        //         nick: None,
        //         password: None,
        //         extensions: vec![],
        //     },
        // })
        // .await?;
        Ok(())
    }

    pub(super) async fn handle_changed_bookmarks(
        &self,
        bookmarks: Vec<ConferenceBookmark>,
    ) -> Result<()> {
        Ok(())
    }

    pub(super) async fn handle_published_bookmarks2(
        &self,
        bookmarks: Vec<ConferenceBookmark>,
    ) -> Result<()> {
        Ok(())
    }

    pub(super) async fn handle_retracted_bookmarks2(&self, jids: Vec<Jid>) -> Result<()> {
        Ok(())
    }
}

#[cfg(feature = "debug")]
impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub async fn load_bookmarks(&self) -> Result<Vec<ConferenceBookmark>> {
        let bookmarks_mod = self.client.get_mod::<mods::Bookmark>();
        bookmarks_mod.load_bookmarks().await
    }

    pub async fn load_bookmarks2(&self) -> Result<Vec<ConferenceBookmark>> {
        let bookmarks_mod = self.client.get_mod::<mods::Bookmark2>();
        bookmarks_mod.load_bookmarks().await
    }

    pub async fn destroy_room(&self, room_jid: &BareJid) -> Result<()> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        muc_mod.destroy_room(room_jid).await?;
        Ok(())
    }

    pub async fn delete_bookmark(&self, jid: &Jid) -> Result<()> {
        self.remove_and_publish_bookmark(&jid.to_bare()).await
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    fn muc_service(&self) -> Result<muc::Service, MUCError> {
        let Some(service) = self.inner.muc_service.read().clone() else {
            return Err(MUCError::Unsupported);
        };
        return Ok(service);
    }

    async fn enter_room_if_needed(
        &self,
        room_jid: &BareJid,
        nickname: Option<&str>,
        password: Option<&str>,
    ) -> Result<(), RequestError> {
        // If we're already connected there's nothing to do.
        if self.inner.connected_rooms.read().contains_key(room_jid) {
            return Ok(());
        }

        let room = self.enter_room(room_jid, nickname, password).await?;
        self.did_enter_room(room);

        Ok(())
    }

    async fn enter_room(
        &self,
        room_jid: &BareJid,
        nickname: Option<&str>,
        password: Option<&str>,
    ) -> Result<Room, RequestError> {
        info!("Entering room {}…", room_jid);

        let nickname = nickname
            .ok_or(
                self.connected_jid()
                    .map_err(|err| RequestError::Generic {
                        msg: err.to_string(),
                    })?
                    .node_str(),
            )
            .unwrap_or("unknown-user");
        let room_jid_full = room_jid.with_resource_str(nickname)?;

        let muc_mod = self.client.get_mod::<mods::MUC>();
        let occupancy = muc_mod.enter_room(&room_jid_full, password).await?;

        let caps = self.client.get_mod::<mods::Caps>();
        let settings =
            RoomSettings::try_from(caps.query_disco_info(room_jid.clone(), None).await?)?;

        return Ok(RoomMetadata {
            room_jid: room_jid_full,
            occupancy,
            settings,
        }
        .into());
    }

    fn did_enter_room(&self, room: Room) {
        // TODO: Send event to delegate
        self.inner
            .connected_rooms
            .write()
            .insert(room.jid().clone(), room);
    }

    async fn insert_and_publish_bookmark(&self, bookmark: ConferenceBookmark) -> Result<()> {
        let bare_jid = bookmark.jid.to_bare();
        info!("Inserting bookmark {}…", bare_jid);
        let mut bookmarks = self.inner.bookmarks.write();

        let needs_save = bookmarks.bookmarks.get(&bare_jid) != Some(&bookmark);
        let needs_save2 = bookmarks.bookmarks2.get(&bare_jid) != Some(&bookmark);

        bookmarks
            .bookmarks
            .insert(bare_jid.clone(), bookmark.clone());
        bookmarks
            .bookmarks2
            .insert(bare_jid.clone(), bookmark.clone());
        drop(bookmarks);

        if needs_save {
            info!("Publishing old-style bookmarks…");
            let bookmark_mod = self.client.get_mod::<mods::Bookmark>();
            let guard = self.inner.bookmarks.read();
            let bookmarks = guard.bookmarks.values().cloned();
            bookmark_mod.publish_bookmarks(bookmarks).await?;
        }

        if needs_save2 {
            info!("Publishing new-style bookmark…");
            let bookmark_mod = self.client.get_mod::<mods::Bookmark2>();
            bookmark_mod
                .publish_bookmark(bare_jid.into(), bookmark.conference)
                .await?;
        }

        Ok(())
    }

    async fn remove_and_publish_bookmark(&self, jid: &BareJid) -> Result<()> {
        info!("Deleting bookmark {}…", jid);
        let mut bookmarks = self.inner.bookmarks.write();

        let needs_save = bookmarks.bookmarks.contains_key(&jid);
        let needs_save2 = bookmarks.bookmarks2.contains_key(&jid);

        bookmarks.bookmarks.remove(&jid);
        bookmarks.bookmarks2.remove(&jid);
        drop(bookmarks);

        if needs_save {
            info!("Publishing old-style bookmarks…");
            let bookmark_mod = self.client.get_mod::<mods::Bookmark>();
            let guard = self.inner.bookmarks.read();
            let bookmarks = guard.bookmarks.values().cloned();
            bookmark_mod.publish_bookmarks(bookmarks).await?;
        }

        if needs_save2 {
            info!("Publishing new-style bookmark…");
            let bookmark_mod = self.client.get_mod::<mods::Bookmark2>();
            bookmark_mod.retract_bookmark(jid.clone().into()).await?;
        }

        Ok(())
    }
}
