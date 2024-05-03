// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{Arc, OnceLock, Weak};

use crate::app::deps::DynRoomFactory;
use crate::app::event_handlers::ClientEventDispatcherTrait;
use crate::client::ClientInner;
use crate::domain::rooms::models::Room;
use crate::{Client, ClientDelegate, ClientEvent, ClientRoomEventType};

pub struct ImmediateClientEventDispatcher {
    client_inner: Arc<OnceLock<Weak<ClientInner>>>,
    room_factory: OnceLock<DynRoomFactory>,
    delegate: Option<Box<dyn ClientDelegate>>,
}

impl ImmediateClientEventDispatcher {
    pub fn new(delegate: Option<Box<dyn ClientDelegate>>) -> Self {
        Self {
            client_inner: Arc::new(Default::default()),
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

impl ClientEventDispatcherTrait for ImmediateClientEventDispatcher {
    fn dispatch_event(&self, event: ClientEvent) {
        self.perform_dispatch_event(event);
    }

    fn dispatch_room_event(&self, room: Room, event: ClientRoomEventType) {
        let room_factory = self
            .room_factory
            .get()
            .expect("RoomFactory was not set on ClientEventDispatcher");

        self.perform_dispatch_event(ClientEvent::RoomChanged {
            room: room_factory.build(room),
            r#type: event,
        });
    }
}

impl ImmediateClientEventDispatcher {
    fn perform_dispatch_event(&self, event: ClientEvent) {
        let Some(delegate) = &self.delegate else {
            return;
        };

        let Some(client_inner) = self
            .client_inner
            .get()
            .expect("ClientInner was not set on ImmediateClientEventDispatcher")
            .upgrade()
        else {
            return;
        };

        delegate.handle_event(Client::from(client_inner), event);
    }
}
