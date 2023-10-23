// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{OnceLock, Weak};

use crate::client::ClientInner;
use crate::{ClientDelegate, ClientEvent};

pub struct ClientEventDispatcher {
    client: OnceLock<Weak<ClientInner>>,
    delegate: Option<Box<dyn ClientDelegate>>,
}

impl ClientEventDispatcher {
    pub fn new(delegate: Option<Box<dyn ClientDelegate>>) -> Self {
        Self {
            client: Default::default(),
            delegate,
        }
    }

    pub fn dispatch_event(&self, event: ClientEvent) {
        let Some(ref delegate) = self.delegate else {
            return;
        };

        let Some(client_inner) = self
            .client
            .get()
            .expect("ClientInner was not set on ClientEventDispatcher")
            .upgrade()
        else {
            return;
        };

        delegate.handle_event(client_inner.into(), event)
    }
}
