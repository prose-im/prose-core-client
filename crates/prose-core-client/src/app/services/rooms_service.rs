// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::atomic::Ordering;

use anyhow::{bail, Result};
use jid::BareJid;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{DynAppContext, DynRoomManagementService, DynSidebarDomainService};
use crate::domain::rooms::models::constants::MAX_PARTICIPANTS_PER_GROUP;
use crate::domain::rooms::services::{CreateOrEnterRoomRequest, CreateRoomType};
use crate::domain::shared::models::RoomId;
use crate::dtos::PublicRoomInfo;

#[derive(InjectDependencies)]
pub struct RoomsService {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    room_management_service: DynRoomManagementService,
    #[inject]
    sidebar_domain_service: DynSidebarDomainService,
}

impl RoomsService {
    #[tracing::instrument(skip(self))]
    pub async fn start_observing_rooms(&self) -> Result<()> {
        if self.ctx.is_observing_rooms.swap(true, Ordering::Acquire) {
            return Ok(());
        }
        self.sidebar_domain_service
            .load_and_extend_items_from_bookmarks()
            .await?;
        Ok(())
    }

    pub async fn load_public_rooms(&self) -> Result<Vec<PublicRoomInfo>> {
        Ok(self
            .room_management_service
            .load_public_rooms(&self.ctx.muc_service()?)
            .await?)
    }

    pub async fn start_conversation(&self, participants: &[BareJid]) -> Result<RoomId> {
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

    pub async fn join_room(&self, room_jid: &RoomId, password: Option<&str>) -> Result<RoomId> {
        self.sidebar_domain_service
            .insert_item_by_creating_or_joining_room(CreateOrEnterRoomRequest::JoinRoom {
                room_jid: room_jid.clone(),
                password: password.map(ToString::to_string),
            })
            .await
    }

    pub async fn create_room_for_direct_message(
        &self,
        participant_jid: &BareJid,
    ) -> Result<RoomId> {
        self.sidebar_domain_service
            .insert_item_by_creating_or_joining_room(CreateOrEnterRoomRequest::JoinDirectMessage {
                participant: participant_jid.clone(),
            })
            .await
    }

    pub async fn create_room_for_group(&self, participants: &[BareJid]) -> Result<RoomId> {
        self.sidebar_domain_service
            .insert_item_by_creating_or_joining_room(CreateOrEnterRoomRequest::Create {
                service: self.ctx.muc_service()?,
                room_type: CreateRoomType::Group {
                    participants: participants.to_vec(),
                },
            })
            .await
    }

    pub async fn create_room_for_private_channel(
        &self,
        channel_name: impl AsRef<str>,
    ) -> Result<RoomId> {
        self.sidebar_domain_service
            .insert_item_by_creating_or_joining_room(CreateOrEnterRoomRequest::Create {
                service: self.ctx.muc_service()?,
                room_type: CreateRoomType::PrivateChannel {
                    name: channel_name.as_ref().to_string(),
                },
            })
            .await
    }

    pub async fn create_room_for_public_channel(
        &self,
        channel_name: impl AsRef<str>,
    ) -> Result<RoomId> {
        self.sidebar_domain_service
            .insert_item_by_creating_or_joining_room(CreateOrEnterRoomRequest::Create {
                service: self.ctx.muc_service()?,
                room_type: CreateRoomType::PublicChannel {
                    name: channel_name.as_ref().to_string(),
                },
            })
            .await
    }

    pub async fn destroy_room(&self, room_jid: &RoomId) -> Result<()> {
        self.room_management_service
            .destroy_room(room_jid, None)
            .await?;
        Ok(())
    }
}
