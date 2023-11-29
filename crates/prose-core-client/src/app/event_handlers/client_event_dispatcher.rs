// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{Arc, OnceLock, Weak};

use crate::app::deps::DynRoomFactory;
use crate::app::event_handlers::ClientEventDispatcherTrait;
use crate::client::ClientInner;
use crate::domain::rooms::models::RoomInternals;
use crate::{ClientDelegate, ClientEvent, ClientRoomEventType};

pub struct ClientEventDispatcher {
    client_inner: OnceLock<Weak<ClientInner>>,
    room_factory: OnceLock<DynRoomFactory>,
    delegate: Option<Box<dyn ClientDelegate>>,
}

impl ClientEventDispatcher {
    pub fn new(delegate: Option<Box<dyn ClientDelegate>>) -> Self {
        Self {
            client_inner: Default::default(),
            room_factory: Default::default(),
            delegate,
        }
    }

    pub(crate) fn set_client_inner(&self, client_inner: Weak<ClientInner>) {
        self.client_inner
            .set(client_inner)
            .map_err(|_| ())
            .expect("Tried to set client_inner on ClientEventDispatcher more than once");
    }

    pub(crate) fn set_room_factory(&self, factory: DynRoomFactory) {
        self.room_factory
            .set(factory)
            .map_err(|_| ())
            .expect("Tried to set room_factory on ClientEventDispatcher more than once");
    }
}

impl ClientEventDispatcherTrait for ClientEventDispatcher {
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

    fn dispatch_room_event(&self, room: Arc<RoomInternals>, event: ClientRoomEventType) {
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

        let room_factory = self
            .room_factory
            .get()
            .expect("RoomFactory was not set on ClientEventDispatcher");

        delegate.handle_event(
            client_inner.into(),
            ClientEvent::RoomChanged {
                room: room_factory.build(room),
                r#type: event,
            },
        )
    }
}
