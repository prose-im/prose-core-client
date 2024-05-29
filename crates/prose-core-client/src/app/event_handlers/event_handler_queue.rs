// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::OnceLock;

use tracing::error;

use prose_xmpp::Event as XMPPEvent;

use crate::app::event_handlers::{ServerEvent, ServerEventHandler};
use crate::infra::xmpp::event_parser::parse_xmpp_event;

pub struct ServerEventHandlerQueue {
    handlers: OnceLock<Vec<Box<dyn ServerEventHandler>>>,
}

impl ServerEventHandlerQueue {
    pub fn new() -> Self {
        Self {
            handlers: Default::default(),
        }
    }

    pub fn set_handlers(&self, handlers: Vec<Box<dyn ServerEventHandler>>) {
        self.handlers
            .set(handlers)
            .map_err(|_| ())
            .expect("Tried to applied handlers ServerEventHandlerQueue more than once");
    }

    pub async fn handle_event(&self, event: XMPPEvent) {
        let events = match parse_xmpp_event(event) {
            Ok(event) => event,
            Err(err) => {
                error!("Failed to parse XMPP event. Reason: {}", err.to_string());
                return;
            }
        };

        for event in events {
            self.handle_server_event(event).await
        }
    }
}

impl ServerEventHandlerQueue {
    pub async fn handle_server_event(&self, event: ServerEvent) {
        let mut event = event;
        let handlers = self
            .handlers
            .get()
            .expect("Handlers were not set in ServerEventHandlerQueue");

        for handler in handlers.iter() {
            match handler.handle_event(event).await {
                Ok(None) => return,
                Ok(Some(e)) => event = e,
                Err(err) => {
                    error!(
                        "Event handler '{}' aborted with error: {}",
                        handler.name(),
                        err.to_string()
                    );
                    return;
                }
            }
        }
    }
}
