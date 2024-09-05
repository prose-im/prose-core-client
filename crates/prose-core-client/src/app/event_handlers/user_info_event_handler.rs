// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use crate::app::deps::{
    DynAppContext, DynClientEventDispatcher, DynConnectedRoomsReadOnlyRepository,
    DynUserInfoDomainService,
};
use crate::app::event_handlers::{
    ServerEvent, ServerEventHandler, UserInfoEvent, UserInfoEventType,
};
use crate::domain::user_info::models::Avatar;
use crate::dtos::ParticipantId;
use crate::ClientEvent;
use prose_proc_macros::InjectDependencies;

#[derive(InjectDependencies)]
pub struct UserInfoEventHandler {
    #[inject]
    client_event_dispatcher: DynClientEventDispatcher,
    #[inject]
    ctx: DynAppContext,
    #[inject]
    connected_rooms_repo: DynConnectedRoomsReadOnlyRepository,
    #[inject]
    user_info_domain_service: DynUserInfoDomainService,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ServerEventHandler for UserInfoEventHandler {
    fn name(&self) -> &'static str {
        "user_state"
    }

    async fn handle_event(&self, event: ServerEvent) -> Result<Option<ServerEvent>> {
        match event {
            ServerEvent::UserInfo(event) => {
                self.handle_user_info_event(event).await?;
            }
            _ => return Ok(Some(event)),
        }
        Ok(None)
    }
}

impl UserInfoEventHandler {
    async fn handle_user_info_event(&self, event: UserInfoEvent) -> Result<()> {
        match event.r#type {
            UserInfoEventType::AvatarChanged { metadata } => {
                let avatar = Avatar::from_metadata(event.user_id.clone(), metadata);

                self.user_info_domain_service
                    .handle_avatar_changed(&event.user_id, Some(avatar.clone()))
                    .await?;

                if let Some(room) = self
                    .connected_rooms_repo
                    .get(&self.ctx.connected_account()?, event.user_id.as_ref())
                {
                    room.with_participants_mut(|p| {
                        p.set_avatar(&ParticipantId::User(event.user_id), Some(avatar));
                    });
                    self.client_event_dispatcher
                        .dispatch_event(ClientEvent::SidebarChanged);
                }
            }
            UserInfoEventType::ProfileChanged { profile } => {
                self.user_info_domain_service
                    .handle_user_profile_changed(&event.user_id, Some(profile))
                    .await?;
            }
            UserInfoEventType::StatusChanged { status } => {
                self.user_info_domain_service
                    .handle_user_status_changed(&event.user_id, status)
                    .await?;
            }
            UserInfoEventType::NicknameChanged { nickname } => {
                self.user_info_domain_service
                    .handle_nickname_changed(&event.user_id, nickname)
                    .await?;
            }
        }

        Ok(())
    }
}
