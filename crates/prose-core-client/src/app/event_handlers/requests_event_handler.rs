// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use tracing::info;

use prose_proc_macros::InjectDependencies;
use prose_xmpp::mods::{caps, ping, profile, roster};
use prose_xmpp::Event;

use crate::app::deps::{DynAppContext, DynRequestHandlingService, DynTimeProvider};
use crate::app::event_handlers::{XMPPEvent, XMPPEventHandler};
use crate::domain::general::services::SubscriptionResponse;

/// Handles various server requests.
#[derive(InjectDependencies)]
pub(crate) struct RequestsEventHandler {
    #[inject]
    request_handling_service: DynRequestHandlingService,
    #[inject]
    ctx: DynAppContext,
    #[inject]
    time_provider: DynTimeProvider,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl XMPPEventHandler for RequestsEventHandler {
    fn name(&self) -> &'static str {
        "requests"
    }

    async fn handle_event(&self, event: XMPPEvent) -> Result<Option<XMPPEvent>> {
        match event {
            Event::Caps(event) => match event {
                caps::Event::DiscoInfoQuery {
                    from,
                    id,
                    node: _node,
                } => {
                    self.request_handling_service
                        .respond_to_disco_info_query(&from, &id, &self.ctx.capabilities)
                        .await?;
                    Ok(None)
                }
                _ => Ok(Some(Event::Caps(event))),
            },
            Event::Ping(event) => match event {
                ping::Event::Ping { from, id } => {
                    self.request_handling_service
                        .respond_to_ping(&from, &id)
                        .await?;
                    Ok(None)
                }
            },
            Event::Profile(event) => match event {
                profile::Event::EntityTimeQuery { from, id } => {
                    self.request_handling_service
                        .respond_to_entity_time_request(&from, &id, &self.time_provider.now())
                        .await?;
                    Ok(None)
                }
                profile::Event::SoftwareVersionQuery { from, id } => {
                    self.request_handling_service
                        .respond_to_software_version_request(&from, &id, &self.ctx.software_version)
                        .await?;
                    Ok(None)
                }
                profile::Event::LastActivityQuery { from, id } => {
                    self.request_handling_service
                        .respond_to_last_activity_request(&from, &id, 0)
                        .await?;
                    Ok(None)
                }
                _ => Ok(Some(Event::Profile(event))),
            },
            Event::Roster(event) => match event {
                roster::Event::PresenceSubscriptionRequest { from } => {
                    info!("Approving presence subscription request from {}…", from);
                    self.request_handling_service
                        .respond_to_presence_subscription_request(
                            &from,
                            SubscriptionResponse::Approve,
                        )
                        .await?;
                    Ok(None)
                }
            },
            _ => Ok(Some(event)),
        }
    }
}
