// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::DynUserInfoDomainService;
use crate::app::event_handlers::{
    ServerEvent, ServerEventHandler, UserInfoEvent, UserInfoEventType,
};

#[derive(InjectDependencies)]
pub struct UserStateEventHandler {
    #[inject]
    user_info_domain_service: DynUserInfoDomainService,
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
                self.user_info_domain_service
                    .handle_avatar_changed(&event.user_id, Some(&metadata))
                    .await?;
            }
            UserInfoEventType::ProfileChanged { profile } => {
                self.user_info_domain_service
                    .handle_user_profile_changed(&event.user_id, Some(&profile))
                    .await?;
            }
            UserInfoEventType::StatusChanged { status } => {
                self.user_info_domain_service
                    .handle_user_status_changed(&event.user_id, status.as_ref())
                    .await?;
            }
        }

        Ok(())
    }
}
