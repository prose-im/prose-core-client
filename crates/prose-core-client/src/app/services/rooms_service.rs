// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{bail, Result};
use jid::BareJid;
use tracing::info;

use prose_proc_macros::InjectDependencies;
use prose_xmpp::mods;

use crate::app::deps::{
    DynAppContext, DynConnectedRoomsRepository, DynRoomFactory, DynRoomManagementService,
    DynRoomsDomainService,
};
use crate::app::services::RoomEnvelope;
use crate::domain::rooms::services::{
    CreateOrEnterRoomRequest, CreateOrEnterRoomRequestType, CreateRoomType,
};

#[derive(InjectDependencies)]
pub struct RoomsService {
    #[inject]
    connected_rooms_repo: DynConnectedRoomsRepository,
    #[inject]
    room_management_service: DynRoomManagementService,
    #[inject]
    room_factory: DynRoomFactory,
    #[inject]
    ctx: DynAppContext,
    #[inject]
    rooms_domain_service: DynRoomsDomainService,
}

impl RoomsService {
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
