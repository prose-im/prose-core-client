// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::atomic::Ordering;

use anyhow::{bail, Result};
use jid::BareJid;
use tracing::{error, info};

use prose_proc_macros::InjectDependencies;
use prose_xmpp::mods;

use crate::app::deps::{
    DynAppContext, DynBookmarksRepository, DynClientEventDispatcher, DynConnectedRoomsRepository,
    DynContactsRepository, DynRoomFactory, DynRoomManagementService, DynRoomsDomainService,
    DynUserProfileRepository,
};
use crate::app::services::RoomEnvelope;
use crate::domain::rooms::models::RoomInternals;
use crate::domain::rooms::services::{
    CreateOrEnterRoomRequest, CreateOrEnterRoomRequestType, CreateRoomType,
};
use crate::domain::shared::utils::build_contact_name;
use crate::ClientEvent;

#[derive(InjectDependencies)]
pub struct RoomsService {
    #[inject]
    client_event_dispatcher: DynClientEventDispatcher,
    #[inject]
    bookmarks_repo: DynBookmarksRepository,
    #[inject]
    connected_rooms_repo: DynConnectedRoomsRepository,
    #[inject]
    contacts_repo: DynContactsRepository,
    #[inject]
    ctx: DynAppContext,
    #[inject]
    room_management_service: DynRoomManagementService,
    #[inject]
    room_factory: DynRoomFactory,
    #[inject]
    rooms_domain_service: DynRoomsDomainService,
    #[inject]
    user_profile_repo: DynUserProfileRepository,
}

impl RoomsService {
    pub async fn start_observing_rooms(&self) -> Result<()> {
        if self.ctx.is_observing_rooms.swap(true, Ordering::Acquire) {
            return Ok(());
        }

        let user_jid = self.ctx.connected_jid()?;

        // Insert contacts as "Direct Message" rooms…
        let direct_message_rooms = {
            let contacts = self.contacts_repo.get_all(&user_jid.to_bare()).await?;
            let mut rooms = vec![];

            for contact in contacts {
                let user_profile = self
                    .user_profile_repo
                    .get(&contact.jid)
                    .await
                    .ok()
                    .map(|maybe_profile| maybe_profile.unwrap_or_default())
                    .unwrap_or_default();
                let room = RoomInternals::for_direct_message(
                    &user_jid,
                    &contact,
                    &build_contact_name(&contact, &user_profile),
                );
                rooms.push(room)
            }

            rooms
        };
        self.connected_rooms_repo.replace(direct_message_rooms);

        let bookmarks = match self.bookmarks_repo.get_all().await {
            Ok(bookmarks) => bookmarks,
            Err(error) => {
                error!("Failed to load bookmarks. Reason: {}", error.to_string());
                Default::default()
            }
        };

        let mut invalid_bookmarks = vec![];

        for bookmark in bookmarks {
            let result = self
                .rooms_domain_service
                .create_or_join_room(CreateOrEnterRoomRequest {
                    r#type: CreateOrEnterRoomRequestType::Join {
                        room_jid: bookmark.room_jid.clone(),
                        nickname: None,
                        password: None,
                    },
                    save_bookmark: false,
                    notify_delegate: false,
                })
                .await;

            match result {
                Ok(_) => (),
                Err(error) if error.is_gone_err() => {
                    // The room does not exist anymore…
                    invalid_bookmarks.push(bookmark.room_jid);
                }
                Err(error) => error!(
                    "Failed to enter room {}. Reason: {}",
                    bookmark.room_jid,
                    error.to_string()
                ),
            }
        }

        if !invalid_bookmarks.is_empty() {
            info!("Deleting {} invalid bookmarks…", invalid_bookmarks.len());
            if let Err(error) = self.bookmarks_repo.delete(&invalid_bookmarks).await {
                error!(
                    "Failed to delete invalid bookmarks. Reason {}",
                    error.to_string()
                )
            }
        }

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::RoomsChanged);

        Ok(())
    }

    pub fn connected_rooms(&self) -> Vec<RoomEnvelope> {
        self.connected_rooms_repo
            .get_all()
            .into_iter()
            .filter_map(|internals| {
                if internals.is_pending() {
                    None
                } else {
                    Some(self.room_factory.build(internals.clone()))
                }
            })
            .collect()
    }

    pub async fn load_public_rooms(&self) -> Result<Vec<mods::muc::Room>> {
        Ok(self
            .room_management_service
            .load_public_rooms(&self.ctx.muc_service()?)
            .await?)
    }

    pub async fn join_room(
        &self,
        room_jid: &BareJid,
        password: Option<&str>,
    ) -> Result<RoomEnvelope> {
        let room = self
            .rooms_domain_service
            .create_or_join_room(CreateOrEnterRoomRequest {
                r#type: CreateOrEnterRoomRequestType::Join {
                    room_jid: room_jid.clone(),
                    nickname: None,
                    password: password.map(ToString::to_string),
                },
                save_bookmark: true,
                notify_delegate: false,
            })
            .await?;
        Ok(self.room_factory.build(room))
    }

    pub async fn create_room_for_direct_message(
        &self,
        participants: &[BareJid],
    ) -> Result<RoomEnvelope> {
        if participants.is_empty() {
            bail!("Group must have at least one other participant.")
        }

        let room = self
            .rooms_domain_service
            .create_or_join_room(CreateOrEnterRoomRequest {
                r#type: CreateOrEnterRoomRequestType::Create {
                    service: self.ctx.muc_service()?,
                    room_type: CreateRoomType::Group {
                        participants: participants.to_vec(),
                        send_invites: true,
                    },
                },
                save_bookmark: true,
                notify_delegate: false,
            })
            .await?;

        Ok(self.room_factory.build(room))
    }

    pub async fn create_room_for_private_channel(
        &self,
        channel_name: impl AsRef<str>,
    ) -> Result<RoomEnvelope> {
        // Create room…
        info!(
            "Creating private channel with name {}…",
            channel_name.as_ref()
        );

        let room = self
            .rooms_domain_service
            .create_or_join_room(CreateOrEnterRoomRequest {
                r#type: CreateOrEnterRoomRequestType::Create {
                    service: self.ctx.muc_service()?,
                    room_type: CreateRoomType::PrivateChannel {
                        name: channel_name.as_ref().to_string(),
                    },
                },
                save_bookmark: true,
                notify_delegate: false,
            })
            .await?;

        Ok(self.room_factory.build(room))
    }

    pub async fn create_room_for_public_channel(
        &self,
        channel_name: impl AsRef<str>,
    ) -> Result<RoomEnvelope> {
        // Create room…
        info!(
            "Creating public channel with name {}…",
            channel_name.as_ref()
        );

        let room = self
            .rooms_domain_service
            .create_or_join_room(CreateOrEnterRoomRequest {
                r#type: CreateOrEnterRoomRequestType::Create {
                    service: self.ctx.muc_service()?,
                    room_type: CreateRoomType::PublicChannel {
                        name: channel_name.as_ref().to_string(),
                    },
                },
                save_bookmark: true,
                notify_delegate: false,
            })
            .await?;

        Ok(self.room_factory.build(room))
    }

    pub async fn destroy_room(&self, room_jid: &BareJid) -> Result<()> {
        self.room_management_service.destroy_room(room_jid).await?;
        Ok(())
    }
}
