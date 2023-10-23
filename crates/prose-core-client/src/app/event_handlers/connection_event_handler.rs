// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::atomic::Ordering;

use anyhow::Result;
use async_trait::async_trait;

use prose_xmpp::{client, Event};

use crate::app::deps::{DynAppContext, DynAppServiceDependencies};
use crate::app::event_handlers::{XMPPEvent, XMPPEventHandler};
use crate::client_event::ConnectionEvent;
use crate::ClientEvent;

pub struct ConnectionEventHandler {
    ctx: DynAppContext,
    deps: DynAppServiceDependencies,
}

#[async_trait]
impl XMPPEventHandler for ConnectionEventHandler {
    fn name(&self) -> &'static str {
        "connection"
    }

    async fn handle_event(&self, event: XMPPEvent) -> Result<Option<XMPPEvent>> {
        match event {
            Event::Client(event) => match event {
                client::Event::Connected => {
                    // We'll send an event from our `connect` method since we need to gather
                    // information about the server first. Once we'll fire the event SDK consumers
                    // can be sure that we have everything we need.
                    Ok(None)
                }
                client::Event::Disconnected { error } => {
                    self.ctx.is_observing_rooms.store(false, Ordering::Relaxed);
                    self.deps.event_dispatcher.dispatch_event(
                        ClientEvent::ConnectionStatusChanged {
                            event: ConnectionEvent::Disconnect { error },
                        },
                    );
                    Ok(None)
                }
            },
            _ => Ok(Some(event)),
        }
    }
}
