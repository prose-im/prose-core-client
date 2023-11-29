// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use crate::app::event_handlers::{ServerEventHandler, XMPPEvent, XMPPEventHandler};
use crate::parse_xmpp_event;

/// A compatibility layer to help with migrating all event handlers to the new ServerEvent.
pub struct ServerEventHandlerWrapper<T: ServerEventHandler> {
    event_handler: T,
}

impl<T: ServerEventHandler> ServerEventHandlerWrapper<T> {
    pub fn new(event_handler: T) -> Self {
        Self { event_handler }
    }
}

#[async_trait]
impl<T: ServerEventHandler> XMPPEventHandler for ServerEventHandlerWrapper<T> {
    fn name(&self) -> &'static str {
        self.event_handler.name()
    }

    async fn handle_event(&self, event: XMPPEvent) -> Result<Option<XMPPEvent>> {
        let server_events = parse_xmpp_event(event.clone())?;
        for event in server_events {
            self.event_handler.handle_event(event).await?;
        }
        return Ok(Some(event));
    }
}
