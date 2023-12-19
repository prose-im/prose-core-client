// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::atomic::Ordering;

use anyhow::Result;
use async_trait::async_trait;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{DynAppContext, DynClientEventDispatcher};
use crate::app::event_handlers::{ConnectionEvent, ServerEvent, ServerEventHandler};
use crate::{ClientEvent, ConnectionEvent as ClientConnectionEvent};

#[derive(InjectDependencies)]
pub struct ConnectionEventHandler {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    client_event_dispatcher: DynClientEventDispatcher,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ServerEventHandler for ConnectionEventHandler {
    fn name(&self) -> &'static str {
        "connection"
    }

    async fn handle_event(&self, event: ServerEvent) -> Result<Option<ServerEvent>> {
        match event {
            ServerEvent::Connection(event) => self.handle_connection_event(event).await?,
            _ => return Ok(Some(event)),
        }
        Ok(None)
    }
}

impl ConnectionEventHandler {
    async fn handle_connection_event(&self, event: ConnectionEvent) -> Result<()> {
        match event {
            ConnectionEvent::Connected => {
                // We'll send an event from our `connect` method since we need to gather
                // information about the server first. Once we'll fire the event SDK consumers
                // can be sure that we have everything we need.
            }
            ConnectionEvent::Disconnected { error } => {
                self.ctx.is_observing_rooms.store(false, Ordering::Relaxed);
                self.client_event_dispatcher
                    .dispatch_event(ClientEvent::ConnectionStatusChanged {
                        event: ClientConnectionEvent::Disconnect { error },
                    });
            }
        }
        Ok(())
    }
}
