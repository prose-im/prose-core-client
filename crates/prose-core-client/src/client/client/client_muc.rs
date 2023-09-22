// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{bail, Result};
use jid::{BareJid, Jid};
use prose_xmpp::stanza::muc::{mediated_invite, DirectInvite, MediatedInvite};
use prose_xmpp::stanza::ConferenceBookmark;
use prose_xmpp::{mods, RequestError};
use std::collections::HashSet;
use std::iter;
use tracing::info;
use xmpp_parsers::bookmarks2::{Autojoin, Conference};
use xmpp_parsers::muc::user::Affiliation;

use crate::avatar_cache::AvatarCache;
use crate::client::room::RoomEnvelope;
use crate::data_cache::DataCache;
use crate::types::muc::{BookmarkMetadata, RoomMetadata, RoomSettings};
use crate::types::{muc, Bookmarks, ConnectedRoom};
use crate::util::StringExt;

use super::Client;

#[derive(thiserror::Error, Debug, PartialEq)]
enum MUCError {
    #[error("Server does not support MUC (XEP-0045)")]
    Unsupported,
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub fn connected_rooms(&self) -> Vec<ConnectedRoom<D, A>> {
        self.inner
            .connected_rooms
            .read()
            .values()
            .filter_map(|envelope| match envelope {
                RoomEnvelope::Pending(_) => None,
                RoomEnvelope::DirectMessage(room) => {
                    Some(ConnectedRoom::DirectMessage(room.clone()))
                }
                RoomEnvelope::Group(room) => Some(ConnectedRoom::Group(room.clone())),
                RoomEnvelope::PrivateChannel(room) => {
                    Some(ConnectedRoom::PrivateChannel(room.clone()))
                }
                RoomEnvelope::PublicChannel(room) => {
                    Some(ConnectedRoom::PublicChannel(room.clone()))
                }
                RoomEnvelope::Generic(room) => Some(ConnectedRoom::Generic(room.clone())),
            })
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
        let user_jid = self.connected_jid()?.into_bare();
        let mut participant_names = vec![];

        for jid in participants.iter().chain(iter::once(&user_jid)) {
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
        let room = RoomEnvelope::from((metadata, user_jid, self));
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
                nick: Some(room.user_nickname().to_string()),
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

    pub async fn create_private_channel(&self, channel_name: impl AsRef<str>) -> Result<()> {
        // Create room…
        info!(
            "Creating private channel with name {}…",
            channel_name.as_ref()
        );

        let room: RoomEnvelope<D, A> = (
            self.muc_service()?
                .create_or_join_private_channel(channel_name.as_ref())
                .await?,
            self.connected_jid()?.into_bare(),
            self,
        )
            .try_into()?;

        self.finish_create_channel(channel_name.as_ref(), room)
            .await
    }

    pub async fn create_public_channel(&self, channel_name: impl AsRef<str>) -> Result<()> {
        // Create room…
        info!(
            "Creating public channel with name {}…",
            channel_name.as_ref()
        );

        let room: RoomEnvelope<D, A> = (
            self.muc_service()?
                .create_or_join_public_channel(channel_name.as_ref())
                .await?,
            self.connected_jid()?.into_bare(),
            self,
        )
            .into();

        self.finish_create_channel(channel_name.as_ref(), room)
            .await
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub(super) async fn load_bookmarks(&self) -> Result<Bookmarks> {
        let bookmark_mod = self.client.get_mod::<mods::Bookmark>();
        let bookmark2_mod = self.client.get_mod::<mods::Bookmark2>();

        let bookmarks = Bookmarks::new(
            bookmark_mod.load_bookmarks().await?,
            bookmark2_mod.load_bookmarks().await?,
        );

        Ok(bookmarks)
    }

    pub(super) async fn handle_direct_invite(
        &self,
        _from: Jid,
        _invite: DirectInvite,
    ) -> Result<()> {
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
        _bookmarks: Vec<ConferenceBookmark>,
    ) -> Result<()> {
        Ok(())
    }

    pub(super) async fn handle_published_bookmarks2(
        &self,
        _bookmarks: Vec<ConferenceBookmark>,
    ) -> Result<()> {
        Ok(())
    }

    pub(super) async fn handle_retracted_bookmarks2(&self, _jids: Vec<Jid>) -> Result<()> {
        Ok(())
    }

    pub(super) async fn enter_room(
        &self,
        room_jid: &BareJid,
        nickname: Option<&str>,
        password: Option<&str>,
    ) -> Result<(), RequestError> {
        info!("Entering room {}…", room_jid);

        let user_jid = self.connected_jid().map_err(|err| RequestError::Generic {
            msg: err.to_string(),
        })?;
        let nickname = nickname.or(user_jid.node_str()).unwrap_or("unknown-user");

        // Insert pending room so that we don't miss any stanzas for this room while we're
        // connecting to it…
        self.inner.connected_rooms.write().insert(
            room_jid.clone(),
            RoomEnvelope::pending(room_jid, &user_jid.to_bare(), nickname, self),
        );

        let metadata = match self.perform_enter_room(room_jid, nickname, password).await {
            Ok(metadata) => metadata,
            Err(error) => {
                // Remove pending room again…
                self.inner.connected_rooms.write().remove(room_jid);
                return Err(error);
            }
        };

        let mut connected_rooms = self.inner.connected_rooms.write();
        let Some(room) = connected_rooms.remove(room_jid) else {
            return Err(RequestError::Generic {
                msg: "Room was modified during connection".to_string(),
            });
        };

        let room =
            room.promote_to_permanent_room(metadata)
                .map_err(|err| RequestError::Generic {
                    msg: err.to_string(),
                })?;
        connected_rooms.insert(room_jid.clone(), room);
        Ok(())
    }

    pub(super) async fn remove_and_publish_bookmarks(&self, jids: &[BareJid]) -> Result<()> {
        if jids.is_empty() {
            return Ok(());
        }

        info!("Deleting {} bookmarks…", jids.len());
        let mut bookmarks = self.inner.bookmarks.write();
        let bookmarks_to_delete = jids.iter().collect::<HashSet<_>>();

        let bookmarks_len = bookmarks.bookmarks.len();
        let bookmarks2_len = bookmarks.bookmarks.len();

        bookmarks
            .bookmarks
            .retain(|jid, _| !bookmarks_to_delete.contains(jid));
        bookmarks
            .bookmarks2
            .retain(|jid, _| !bookmarks_to_delete.contains(jid));

        let needs_save = bookmarks.bookmarks.len() != bookmarks_len;
        let needs_save2 = bookmarks.bookmarks2.len() != bookmarks2_len;
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
            for jid in jids {
                bookmark_mod.retract_bookmark(jid.clone().into()).await?;
            }
        }

        Ok(())
    }
}

#[cfg(feature = "debug")]
impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub async fn load_bookmarks_dbg(
        &self,
    ) -> Result<(Vec<ConferenceBookmark>, Vec<ConferenceBookmark>)> {
        let bookmarks = self.load_bookmarks().await?;
        Ok((
            bookmarks.bookmarks.values().cloned().collect(),
            bookmarks.bookmarks2.values().cloned().collect(),
        ))
    }

    pub async fn destroy_room(&self, room_jid: &BareJid) -> Result<()> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        muc_mod.destroy_room(room_jid).await?;
        Ok(())
    }

    pub async fn delete_bookmark(&self, jid: &Jid) -> Result<()> {
        self.remove_and_publish_bookmarks(&[jid.to_bare()]).await
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    fn muc_service(&self) -> Result<muc::Service, MUCError> {
        let Some(service) = self.inner.muc_service.read().clone() else {
            return Err(MUCError::Unsupported);
        };
        return Ok(service);
    }

    async fn finish_create_channel(&self, name: &str, room: RoomEnvelope<D, A>) -> Result<()> {
        let bookmark = ConferenceBookmark {
            jid: room.jid().clone().into(),
            conference: Conference {
                autojoin: Autojoin::True,
                name: Some(name.to_string()),
                nick: Some(room.user_nickname().to_string()),
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

    pub(super) async fn perform_enter_room(
        &self,
        room_jid: &BareJid,
        nickname: &str,
        password: Option<&str>,
    ) -> Result<RoomMetadata, RequestError> {
        let room_jid_full = room_jid.with_resource_str(nickname)?;

        let muc_mod = self.client.get_mod::<mods::MUC>();
        let occupancy = muc_mod.enter_room(&room_jid_full, password).await?;

        let caps = self.client.get_mod::<mods::Caps>();
        let settings =
            RoomSettings::try_from(caps.query_disco_info(room_jid.clone(), None).await?)?;

        // When creating a group we change all "members" to "owners", so at least for Prose groups
        // this should work as expected…
        let members = muc_mod
            .request_users(room_jid, Affiliation::Owner)
            .await?
            .into_iter()
            .map(|user| user.jid.into_bare())
            .collect::<Vec<_>>();

        Ok(RoomMetadata {
            room_jid: room_jid_full,
            occupancy,
            settings,
            members,
        })
    }

    fn did_enter_room(&self, room: RoomEnvelope<D, A>) {
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
}
