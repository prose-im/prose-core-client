// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{anyhow, Result};
use jid::{BareJid, Jid};
use prose_xmpp::mods;
use prose_xmpp::stanza::muc::{mediated_invite, DirectInvite, MediatedInvite};
use prose_xmpp::stanza::ConferenceBookmark;
use std::iter;
use tracing::info;
use xmpp_parsers::bookmarks2::{Autojoin, Conference};

use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::types::muc::{BookmarkMetadata, CreateRoomResult, RoomConfig};
use crate::types::{muc, Bookmarks};

use super::Client;

#[derive(thiserror::Error, Debug, PartialEq)]
enum MUCError {
    #[error("Server does not support MUC (XEP-0045)")]
    Unsupported,
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub async fn load_public_rooms(&self) -> Result<Vec<mods::muc::Room>> {
        self.muc_service()?.load_public_rooms().await
    }

    pub async fn create_group(&self, participants: &[BareJid]) -> Result<()> {
        let user_jid = self.connected_jid()?.into_bare();
        let group_name = muc::Service::group_name_for_participants(
            participants.into_iter().chain(iter::once(&user_jid)),
        );

        // Create room…
        info!("Creating group with name {}…", group_name);
        let result = self
            .muc_service()?
            .create_room_with_config(group_name, RoomConfig::group())
            .await;

        let (room_jid, is_new_room) = match result {
            CreateRoomResult::Created(room_jid) => (room_jid, true),
            CreateRoomResult::Joined(room_jid) => (room_jid, false),
            CreateRoomResult::Err(err) => return Err(anyhow!(err)),
        };

        // Save bookmark for created (or joined) room…
        self.save_and_publish_bookmark(ConferenceBookmark {
            jid: room_jid.to_bare().into(),
            conference: Conference {
                autojoin: Autojoin::True,
                name: None,
                nick: Some(room_jid.resource_str().to_string()),
                password: None,
                extensions: vec![BookmarkMetadata {
                    room_type: muc::RoomType::Group,
                    participants: Some(participants.iter().map(Clone::clone).collect()),
                }
                .into()],
            },
        })
        .await?;

        if !is_new_room {
            return Ok(());
        }

        // Send invites…
        info!("Sending invites for created group…");
        let muc_mod = self.client.get_mod::<mods::MUC>();
        muc_mod
            .send_mediated_invite(
                &room_jid.to_bare(),
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
        // self.muc_service()?
        //     .create_room_with_config(
        //         channel_name.as_ref(),
        //         RoomConfig::public_channel(channel_name.as_ref()),
        //     )
        //     .await?;
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
            self.connect_to_room(
                &bookmark.jid.to_bare(),
                bookmark
                    .conference
                    .nick
                    .as_deref()
                    .or(self.connected_jid()?.node_str())
                    .unwrap_or("unknown-user"),
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

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    fn muc_service(&self) -> Result<muc::Service, MUCError> {
        let Some(service) = self.inner.muc_service.read().clone() else {
            return Err(MUCError::Unsupported);
        };
        return Ok(service);
    }

    async fn connect_to_room(
        &self,
        room_jid: &BareJid,
        nickname: impl AsRef<str>,
        password: Option<&str>,
    ) -> Result<()> {
        if !self.inner.connected_rooms.write().insert(room_jid.clone()) {
            return Ok(());
        }

        info!("Entering room {}…", room_jid);
        let muc_mod = self.client.get_mod::<mods::MUC>();
        muc_mod
            .enter_room(room_jid, nickname.as_ref(), password)
            .await?;

        //let caps = self.client.get_mod::<mods::Caps>();
        //let response = caps.query_disco_info(room_jid.clone(), None).await?;

        Ok(())
    }

    async fn save_and_publish_bookmark(&self, bookmark: ConferenceBookmark) -> Result<()> {
        let bare_jid = bookmark.jid.to_bare();
        let mut bookmarks = self.inner.bookmarks.write();

        let save_bookmarks = !bookmarks.bookmarks.contains_key(&bare_jid);
        let save_bookmarks2 = !bookmarks.bookmarks2.contains_key(&bare_jid);

        bookmarks
            .bookmarks
            .insert(bare_jid.clone(), bookmark.clone());
        bookmarks
            .bookmarks2
            .insert(bare_jid.clone(), bookmark.clone());
        drop(bookmarks);

        if save_bookmarks {
            info!("Publishing old-style bookmarks…");
            let bookmark_mod = self.client.get_mod::<mods::Bookmark>();
            let bookmarks = self.inner.bookmarks.read();

            bookmark_mod
                .publish_bookmarks(bookmarks.bookmarks.values().cloned())
                .await?;
        }

        if save_bookmarks2 {
            info!("Publishing new-style bookmark…");
            let bookmark_mod = self.client.get_mod::<mods::Bookmark2>();
            bookmark_mod
                .publish_bookmark(bare_jid.into(), bookmark.conference)
                .await?;
        }

        Ok(())
    }
}
