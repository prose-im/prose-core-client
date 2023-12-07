// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use tracing::info;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{DynAppContext, DynRequestHandlingService, DynTimeProvider};
use crate::app::event_handlers::{RequestEvent, RequestEventType, ServerEvent, ServerEventHandler};
use crate::domain::general::services::SubscriptionResponse;

/// Handles various server requests.
#[derive(InjectDependencies)]
pub struct RequestsEventHandler {
    #[inject]
    request_handling_service: DynRequestHandlingService,
    #[inject]
    ctx: DynAppContext,
    #[inject]
    time_provider: DynTimeProvider,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ServerEventHandler for RequestsEventHandler {
    fn name(&self) -> &'static str {
        "requests"
    }

    async fn handle_event(&self, event: ServerEvent) -> Result<Option<ServerEvent>> {
        match event {
            ServerEvent::Request(event) => {
                self.handle_request_event(event).await?;
            }
            _ => return Ok(Some(event)),
        }
        Ok(None)
    }
}

impl RequestsEventHandler {
    async fn handle_request_event(&self, event: RequestEvent) -> Result<()> {
        match event.r#type {
            RequestEventType::Ping => {
                self.request_handling_service
                    .respond_to_ping(&event.sender_id, &event.request_id)
                    .await?;
            }
            RequestEventType::LocalTime => {
                self.request_handling_service
                    .respond_to_entity_time_request(
                        &event.sender_id,
                        &event.request_id,
                        &self.time_provider.now(),
                    )
                    .await?;
            }
            RequestEventType::LastActivity => {
                self.request_handling_service
                    .respond_to_last_activity_request(&event.sender_id, &event.request_id, 0)
                    .await?;
            }
            RequestEventType::Capabilities { id: _id } => {
                self.request_handling_service
                    .respond_to_disco_info_query(
                        &event.sender_id,
                        &event.request_id,
                        &self.ctx.capabilities,
                    )
                    .await?;
            }
            RequestEventType::SoftwareVersion => {
                self.request_handling_service
                    .respond_to_software_version_request(
                        &event.sender_id,
                        &event.request_id,
                        &self.ctx.software_version,
                    )
                    .await?;
            }
            RequestEventType::PresenceSubscription => {
                info!(
                    "Approving presence subscription request from {}…",
                    event.sender_id
                );
                self.request_handling_service
                    .respond_to_presence_subscription_request(
                        &event.sender_id,
                        SubscriptionResponse::Approve,
                    )
                    .await?;
            }
        }
        Ok(())
    }
}
