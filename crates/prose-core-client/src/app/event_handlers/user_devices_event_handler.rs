// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::DynEncryptionDomainService;
use crate::app::event_handlers::{
    PubSubEventType, ServerEvent, ServerEventHandler, UserDeviceEvent,
};

#[derive(InjectDependencies)]
pub struct UserDevicesEventHandler {
    #[inject]
    encryption_domain_service: DynEncryptionDomainService,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ServerEventHandler for UserDevicesEventHandler {
    fn name(&self) -> &'static str {
        "user_devices"
    }

    async fn handle_event(&self, event: ServerEvent) -> Result<Option<ServerEvent>> {
        match event {
            ServerEvent::UserDevice(event) => self.handle_user_device_event(event).await?,
            _ => return Ok(Some(event)),
        }
        Ok(None)
    }
}

impl UserDevicesEventHandler {
    async fn handle_user_device_event(&self, event: UserDeviceEvent) -> Result<()> {
        match event.r#type {
            PubSubEventType::AddedOrUpdated { items } => {
                for device_list in items {
                    self.encryption_domain_service
                        .handle_received_device_list(&event.user_id, device_list)
                        .await?;
                }
            }
            PubSubEventType::Deleted { .. } => {}
            PubSubEventType::Purged => {}
        }

        Ok(())
    }
}
