// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::OnceLock;

use tracing::error;

use crate::app::event_handlers::{XMPPEvent, XMPPEventHandler};

pub struct XMPPEventHandlerQueue {
    handlers: OnceLock<Vec<Box<dyn XMPPEventHandler>>>,
}

impl XMPPEventHandlerQueue {
    pub fn new() -> Self {
        Self {
            handlers: Default::default(),
        }
    }

    pub fn set_handlers(&self, handlers: Vec<Box<dyn XMPPEventHandler>>) {
        self.handlers
            .set(handlers)
            .map_err(|_| ())
            .expect("Tried to applied handlers XMPPEventHandlerQueue more than once");
    }

    pub async fn handle_event(&self, event: XMPPEvent) {
        let mut event = event;
        let handlers = self
            .handlers
            .get()
            .expect("Handlers were not set in XMPPEventHandlerQueue");

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
