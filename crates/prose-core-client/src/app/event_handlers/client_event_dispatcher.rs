// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{OnceLock, Weak};

use crate::app::event_handlers::EventDispatcher;
use crate::client::ClientInner;
use crate::{ClientDelegate, ClientEvent};

pub struct ClientEventDispatcher {
    client_inner: OnceLock<Weak<ClientInner>>,
    delegate: Option<Box<dyn ClientDelegate>>,
}

impl ClientEventDispatcher {
    pub fn new(delegate: Option<Box<dyn ClientDelegate>>) -> Self {
        Self {
            client_inner: Default::default(),
            delegate,
        }
    }

    pub(crate) fn set_client_inner(&self, client_inner: Weak<ClientInner>) {
        self.client_inner
            .set(client_inner)
            .map_err(|_| ())
            .expect("Tried to set client_inner on ClientEventDispatcher more than once");
    }
}

impl EventDispatcher<ClientEvent> for ClientEventDispatcher {
    fn dispatch_event(&self, event: ClientEvent) {
        let Some(ref delegate) = self.delegate else {
            return;
        };

        let Some(client_inner) = self
            .client_inner
            .get()
            .expect("ClientInner was not set on ClientEventDispatcher")
            .upgrade()
        else {
            return;
        };

        delegate.handle_event(client_inner.into(), event)
    }
}
