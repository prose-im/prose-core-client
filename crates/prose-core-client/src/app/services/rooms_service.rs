// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::atomic::Ordering;
use std::sync::Arc;

use anyhow::{bail, Result};
use jid::BareJid;
use tracing::{error, info};

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{
    DynAppContext, DynBookmarksService, DynClientEventDispatcher, DynConnectedRoomsRepository,
    DynRoomFactory, DynRoomManagementService, DynRoomsDomainService, DynSidebarRepository,
    DynUserProfileRepository,
};
use crate::app::services::RoomEnvelope;
use crate::domain::rooms::models::constants::MAX_PARTICIPANTS_PER_GROUP;
use crate::domain::rooms::models::RoomInternals;
use crate::domain::rooms::services::{
    CreateOrEnterRoomRequest, CreateOrEnterRoomRequestType, CreateRoomType,
};
use crate::domain::shared::models::RoomJid;
use crate::domain::shared::utils::build_contact_name;
use crate::domain::sidebar::models::{Bookmark, BookmarkType, SidebarItem};
use crate::dtos::PublicRoomInfo;
use crate::ClientEvent;

#[derive(thiserror::Error, Debug)]
pub enum RoomError {
    #[error("A room with the chosen name exists already.")]
    Conflict,
}

#[derive(InjectDependencies)]
pub struct RoomsService {
    #[inject]
    client_event_dispatcher: DynClientEventDispatcher,
    #[inject]
    bookmarks_service: DynBookmarksService,
    #[inject]
    connected_rooms_repo: DynConnectedRoomsRepository,
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
    #[inject]
    sidebar_repo: DynSidebarRepository,
}

impl RoomsService {
    pub async fn start_observing_rooms(&self) -> Result<()> {
        if self.ctx.is_observing_rooms.swap(true, Ordering::Acquire) {
            return Ok(());
        }

        let bookmarks = match self.bookmarks_service.load_bookmarks().await {
            Ok(bookmarks) => bookmarks,
            Err(error) => {
                error!("Failed to load bookmarks. Reason: {}", error.to_string());
                Default::default()
            }
        };

        let user_jid = self.ctx.connected_jid()?.into_bare();
        let mut sidebar_items = vec![];

        for bookmark in bookmarks {
            let should_connect = match bookmark.r#type {
                BookmarkType::DirectMessage => false,
                // While our user can remove a Group from their sidebar they should always receive
                // messages from it. In these cases the Group will automatically reappear in the
                // sidebar. We want our users to think about Groups as if they were a
                // Direct Message.
                BookmarkType::Group => true,
                BookmarkType::PublicChannel | BookmarkType::PrivateChannel => bookmark.in_sidebar,
            };

            let mut sidebar_item = SidebarItem {
                name: bookmark.name,
                jid: bookmark.jid,
                r#type: bookmark.r#type,
                is_favorite: bookmark.is_favorite,
                error: None,
            };

            if should_connect {
                let result = self
                    .rooms_domain_service
                    .create_or_join_room(CreateOrEnterRoomRequest {
                        r#type: CreateOrEnterRoomRequestType::Join {
                            room_jid: sidebar_item.jid.clone(),
                            nickname: None,
                            password: None,
                        },
                        save_bookmark: false,
                        insert_sidebar_item: false,
                        notify_delegate: false,
                    })
                    .await;

                match result {
                    Ok(room) => {
                        sidebar_item.name =
                            room.state.read().name.clone().unwrap_or(sidebar_item.name)
                    }
                    Err(error) => sidebar_item.error = Some(error.to_string()),
                }
            }

            if sidebar_item.r#type == BookmarkType::DirectMessage && bookmark.in_sidebar {
                let room = Arc::new(RoomInternals::for_direct_message(
                    &user_jid,
                    &sidebar_item.jid,
                    &sidebar_item.name,
                ));
                _ = self.connected_rooms_repo.set(room);
            }

            if bookmark.in_sidebar {
                sidebar_items.push(sidebar_item)
            }
        }

        self.sidebar_repo.set_all(sidebar_items);

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

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

    pub async fn load_public_rooms(&self) -> Result<Vec<PublicRoomInfo>> {
        Ok(self
            .room_management_service
            .load_public_rooms(&self.ctx.muc_service()?)
            .await?)
    }

    pub async fn start_conversation(&self, participants: &[BareJid]) -> Result<RoomEnvelope> {
        if participants.is_empty() {
            bail!("You need at least one participant to start a conversation")
        }

        match participants.len() {
            0 => bail!("You need at least one participant to start a conversation"),
            1 => self.create_room_for_direct_message(&participants[0]).await,
            2..=MAX_PARTICIPANTS_PER_GROUP => self.create_room_for_group(participants).await,
            _ => bail!("You can't start a simple conversation with more than {} participants. Consider creating a private or a public room instead.", MAX_PARTICIPANTS_PER_GROUP)
        }
    }

    pub async fn join_room(
        &self,
        room_jid: &RoomJid,
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
                insert_sidebar_item: true,
                notify_delegate: false,
            })
            .await?;
        Ok(self.room_factory.build(room))
    }

    pub async fn create_room_for_direct_message(
        &self,
        participant_jid: &BareJid,
    ) -> Result<RoomEnvelope> {
        if let Some(room) = self
            .connected_rooms_repo
            .get(&participant_jid.clone().into())
        {
            return Ok(self.room_factory.build(room));
        }

        let user_profile = self
            .user_profile_repo
            .get(participant_jid)
            .await
            .ok()
            .map(|maybe_profile| maybe_profile.unwrap_or_default())
            .unwrap_or_default();

        let user_jid = self.ctx.connected_jid()?.into_bare();
        let contact_name = build_contact_name(&participant_jid, &user_profile);

        let room = Arc::new(RoomInternals::for_direct_message(
            &user_jid,
            &participant_jid,
            &contact_name,
        ));

        let bookmark = Bookmark {
            name: contact_name.clone(),
            jid: participant_jid.clone().into(),
            r#type: BookmarkType::DirectMessage,
            is_favorite: false,
            in_sidebar: true,
        };
        self.bookmarks_service.save_bookmark(&bookmark).await?;

        let sidebar_item = SidebarItem {
            name: contact_name,
            jid: participant_jid.clone().into(),
            r#type: BookmarkType::DirectMessage,
            is_favorite: false,
            error: None,
        };
        self.sidebar_repo.put(&sidebar_item);

        // We'll ignore the potential error since we've already checked if the room exists already.
        _ = self.connected_rooms_repo.set(room.clone());

        Ok(self.room_factory.build(room))
    }

    pub async fn create_room_for_group(&self, participants: &[BareJid]) -> Result<RoomEnvelope> {
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
                insert_sidebar_item: true,
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
                insert_sidebar_item: true,
                notify_delegate: false,
            })
            .await?;

        Ok(self.room_factory.build(room))
    }

    pub async fn create_room_for_public_channel(
        &self,
        channel_name: impl AsRef<str>,
    ) -> Result<RoomEnvelope> {
        let available_rooms = self
            .room_management_service
            .load_public_rooms(&self.ctx.muc_service()?)
            .await?;

        let lowercase_channel_name = channel_name.as_ref().to_lowercase();
        for room in available_rooms {
            let Some(mut room_name) = room.name else {
                continue;
            };
            room_name.make_ascii_lowercase();
            if room_name == lowercase_channel_name {
                return Err(RoomError::Conflict.into());
            }
        }

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
                insert_sidebar_item: true,
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
