// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use parking_lot::Mutex;
use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;

use pretty_assertions::assert_eq;

use prose_core_client::dtos::{RoomEnvelope, RoomId};
use prose_core_client::test::ConstantTimeProvider;
use prose_core_client::{Client, ClientDelegate, ClientEvent, FsAvatarCache};
use prose_xmpp::test::IncrementingIDProvider;
use prose_xmpp::IDProvider;

use crate::tests::store;

use super::{connector::Connector, test_message_queue::TestMessageQueue};

pub struct TestClient {
    pub(super) client: Client,
    id_provider: IncrementingIDProvider,
    messages: TestMessageQueue,
    context: Mutex<Vec<HashMap<String, String>>>,
}

impl TestClient {
    pub async fn new() -> Self {
        let messages = TestMessageQueue::default();
        let path = tempfile::tempdir().unwrap().path().join("avatars");

        let client = Client::builder()
            .set_connector_provider(Connector::provider(messages.clone()))
            .set_id_provider(IncrementingIDProvider::new("id"))
            .set_avatar_cache(FsAvatarCache::new(&path).unwrap())
            .set_store(store().await.expect("Failed to set up store."))
            .set_time_provider(ConstantTimeProvider::ymd(2024, 02, 19))
            .set_delegate(Some(Box::new(Delegate {
                messages: messages.clone(),
            })))
            .build();

        Self {
            client,
            // We'll just mirror the used ID provider here…
            id_provider: IncrementingIDProvider::new("id"),
            messages,
            context: Default::default(),
        }
    }
}

impl TestClient {
    pub fn send(&self, xml: impl Into<String>) {
        let mut xml = xml.into();

        // Only increase the ID counter if the message contains an ID…
        if xml.contains("{{ID}}") {
            self.push_ctx([("ID".into(), self.id_provider.new_id())].into());
            self.apply_ctx(&mut xml);
            self.pop_ctx();
        } else {
            self.apply_ctx(&mut xml);
        }
        self.messages.send(xml);
    }

    pub fn receive(&self, xml: impl Into<String>) {
        let mut xml = xml.into();

        self.push_ctx([("ID".into(), self.id_provider.last_id())].into());
        self.apply_ctx(&mut xml);
        self.pop_ctx();

        self.messages.receive(xml);
    }

    pub fn event(&self, event: ClientEvent) {
        self.messages.event(event);
    }
}

impl TestClient {
    pub fn push_ctx(&self, ctx: HashMap<String, String>) {
        self.context.lock().push(ctx);
    }

    pub fn pop_ctx(&self) {
        self.context.lock().pop();
    }

    fn apply_ctx(&self, xml_str: &mut String) {
        let guard = self.context.lock();
        for ctx in guard.iter().rev() {
            for (key, value) in ctx {
                *xml_str = xml_str.replace(&format!("{{{{{}}}}}", key.to_uppercase()), value);
            }
        }
    }
}

// impl TestClient {
//     pub async fn get_room(&self, id: impl AsRef<str>) -> RoomEnvelope {
//         let room_id = RoomId::from_str(id.as_ref()).expect("Could not parse room id");
//
//         let Some(item) = self
//             .client
//             .sidebar
//             .sidebar_items()
//             .await
//             .into_iter()
//             .find(|item| item.room.to_generic_room().jid() == &room_id)
//         else {
//             panic!("Could not find connected room with id {room_id}")
//         };
//
//         item.room
//     }
// }

impl Deref for TestClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

struct Delegate {
    messages: TestMessageQueue,
}

impl ClientDelegate for Delegate {
    fn handle_event(&self, _client: Client, received_event: ClientEvent) {
        let Some(expected_event) = self.messages.pop_event() else {
            panic!("Received unexpected event: {:?}", received_event)
        };
        assert_eq!(expected_event, received_event);
    }
}
