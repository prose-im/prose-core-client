// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{Arc, OnceLock, Weak};
use std::time::Duration;

use futures::StreamExt;
use tokio::sync::mpsc::{channel, Sender};
use tracing::debug;

use prose_wasm_utils::{spawn, ProseStreamExt, ReceiverStream};

use crate::app::deps::DynRoomFactory;
use crate::app::event_handlers::ClientEventDispatcherTrait;
use crate::client::ClientInner;
use crate::domain::rooms::models::Room;
use crate::domain::shared::models::RoomType;
use crate::util::coalesce_client_events;
use crate::{Client, ClientDelegate, ClientEvent, ClientRoomEventType};

pub struct ClientEventDispatcher {
    client_inner: Arc<OnceLock<Weak<ClientInner>>>,
    room_factory: OnceLock<DynRoomFactory>,
    sender: Sender<ClientEvent>,
}

impl ClientEventDispatcher {
    pub fn new(delegate: Option<Box<dyn ClientDelegate>>) -> Self {
        let (tx, rx) = channel(50);

        let mut events_stream = ReceiverStream::new(rx).throttled(Duration::from_millis(200));
        let client_inner = Arc::new(OnceLock::<Weak<ClientInner>>::new());

        if let Some(delegate) = delegate {
            let client_inner = client_inner.clone();
            spawn(async move {
                while let Some(mut events) = events_stream.next().await {
                    let Some(client_inner) = client_inner
                        .get()
                        .expect("ClientInner was not set on ClientEventDispatcher")
                        .upgrade()
                    else {
                        return;
                    };

                    let client = Client::from(client_inner);
                    coalesce_client_events(&mut events);

                    for event in events {
                        debug!(event = ?event, "Dispatching event");
                        delegate.handle_event(client.clone(), event)
                    }
                }
            });
        }

        Self {
            client_inner,
            room_factory: Default::default(),
            sender: tx,
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
        debug!(event = ?event, "Enqueuing event");
        _ = self.sender.try_send(event);
    }

    fn dispatch_room_event(&self, room: Room, event: ClientRoomEventType) {
        // We're not sending events for rooms that are still pending…
        if room.r#type == RoomType::Unknown {
            return;
        }

        let room_factory = self
            .room_factory
            .get()
            .expect("RoomFactory was not set on ClientEventDispatcher");

        debug!(room_id = %room.room_id, event = ?event, "Enqueuing room event");

        _ = self.sender.try_send(ClientEvent::RoomChanged {
            room: room_factory.build(room),
            r#type: event,
        })
    }
}
