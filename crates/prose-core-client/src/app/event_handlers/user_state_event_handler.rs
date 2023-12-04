// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{
    DynAppContext, DynAvatarRepository, DynClientEventDispatcher, DynUserInfoRepository,
    DynUserProfileRepository,
};
use crate::app::event_handlers::{
    ServerEvent, ServerEventHandler, UserInfoEvent, UserInfoEventType,
};
use crate::ClientEvent;

#[derive(InjectDependencies)]
pub struct UserStateEventHandler {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    client_event_dispatcher: DynClientEventDispatcher,
    #[inject]
    avatar_repo: DynAvatarRepository,
    #[inject]
    user_info_repo: DynUserInfoRepository,
    #[inject]
    user_profile_repo: DynUserProfileRepository,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ServerEventHandler for UserStateEventHandler {
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

impl UserStateEventHandler {
    async fn handle_user_info_event(&self, event: UserInfoEvent) -> Result<()> {
        match event.r#type {
            UserInfoEventType::AvatarChanged { metadata } => {
                self.user_info_repo
                    .set_avatar_metadata(&event.user_id, &metadata)
                    .await?;
                self.avatar_repo
                    .precache_avatar_image(&event.user_id, &metadata.to_info())
                    .await?;
                self.client_event_dispatcher
                    .dispatch_event(ClientEvent::AvatarChanged { id: event.user_id });
            }
            UserInfoEventType::ProfileChanged { profile } => {
                self.user_profile_repo.set(&event.user_id, &profile).await?;
                self.client_event_dispatcher
                    .dispatch_event(ClientEvent::ContactChanged { id: event.user_id });
            }
            UserInfoEventType::StatusChanged { status } => {
                self.user_info_repo
                    .set_user_activity(&event.user_id, status.as_ref())
                    .await?;
                self.client_event_dispatcher
                    .dispatch_event(ClientEvent::ContactChanged { id: event.user_id });
            }
        }

        Ok(())
    }
}
