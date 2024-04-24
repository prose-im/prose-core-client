// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use parking_lot::Mutex;
use pretty_assertions::assert_eq;

use prose_core_client::domain::encryption::services::IncrementingUserDeviceIdProvider;
use prose_core_client::dtos::{DeviceId, RoomEnvelope, RoomId};
use prose_core_client::infra::encryption::EncryptionKeysRepository;
use prose_core_client::infra::general::mocks::StepRngProvider;
use prose_core_client::test::ConstantTimeProvider;
use prose_core_client::{Client, ClientDelegate, ClientEvent, FsAvatarCache, SignalServiceHandle};
use prose_xmpp::test::IncrementingIDProvider;
use prose_xmpp::IDProvider;

use crate::tests::store;

use super::{connector::Connector, test_message_queue::TestMessageQueue};

#[allow(dead_code)]
pub struct TestClient {
    pub(super) client: Client,
    connector: Connector,
    id_provider: IncrementingIDProvider,
    pub(super) short_id_provider: IncrementingIDProvider,
    messages: TestMessageQueue,
    context: Mutex<Vec<HashMap<String, String>>>,
}

impl Drop for TestClient {
    fn drop(&mut self) {
        // Don't perform any further check if we're already panicking…
        if std::thread::panicking() {
            return;
        }

        let num_remaining_messages = self.messages.len();
        assert_eq!(
            num_remaining_messages, 0,
            "TestClient dropped while still containing {num_remaining_messages} messages."
        );
    }
}

impl TestClient {
    pub async fn new() -> Self {
        let messages = TestMessageQueue::default();
        let connector = Connector::new(messages.clone());
        let path = tempfile::tempdir().unwrap().path().join("avatars");
        let device_id = DeviceId::from(TestClient::device_id());
        let store = store().await.expect("Failed to set up store.");

        let encryption_service = SignalServiceHandle::new(
            Arc::new(EncryptionKeysRepository::new(store.clone())),
            Arc::new(StepRngProvider::default()),
        );

        let client = Client::builder()
            .set_connector_provider(connector.provider())
            .set_id_provider(IncrementingIDProvider::new("id"))
            .set_short_id_provider(IncrementingIDProvider::new("short-id"))
            .set_rng_provider(StepRngProvider::default())
            .set_avatar_cache(FsAvatarCache::new(&path).unwrap())
            .set_encryption_service(Arc::new(encryption_service))
            .set_store(store)
            .set_time_provider(ConstantTimeProvider::ymd(2024, 02, 19))
            .set_user_device_id_provider(IncrementingUserDeviceIdProvider::new(*device_id.as_ref()))
            .set_delegate(Some(Box::new(Delegate {
                messages: messages.clone(),
            })))
            .build();

        let client = Self {
            client,
            // We'll just mirror the used ID provider here…
            connector,
            id_provider: IncrementingIDProvider::new("id"),
            short_id_provider: IncrementingIDProvider::new("short-id"),
            messages,
            context: Default::default(),
        };

        client.push_ctx([("USER_DEVICE_ID".into(), format!("{}", device_id.as_ref()))].into());

        client
    }
}

#[macro_export]
macro_rules! send(
    ($client:ident, $element:expr) => (
        $client.send($element, file!(), line!())
    )
);

#[macro_export]
macro_rules! recv(
    ($client:ident, $element:expr) => (
        $client.receive($element, file!(), line!())
    )
);

#[macro_export]
macro_rules! event(
    ($client:ident, $event:expr) => (
        $client.event($event, file!(), line!())
    )
);

#[allow(dead_code)]
impl TestClient {
    pub fn send(&self, xml: impl Into<String>, file: &str, line: u32) {
        let mut xml = xml.into();

        // Only increase the ID counter if the message contains an ID…
        if xml.contains("{{ID}}") {
            self.push_ctx([("ID".into(), self.id_provider.new_id())].into());
            self.apply_ctx(&mut xml);
            self.pop_ctx();
        } else {
            self.apply_ctx(&mut xml);
        }
        self.messages.send(xml, file, line);
    }

    pub fn receive(&self, xml: impl Into<String>, file: &str, line: u32) {
        let mut xml = xml.into();

        self.push_ctx([("ID".into(), self.id_provider.last_id())].into());
        self.apply_ctx(&mut xml);
        self.pop_ctx();

        self.messages.receive(xml, file, line);
    }

    pub fn event(&self, event: ClientEvent, file: &str, line: u32) {
        self.messages.event(event, file, line);
    }

    pub async fn receive_next(&self) {
        self.connector.receive_next().await
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

impl TestClient {
    pub async fn get_room(&self, id: impl Into<RoomId>) -> RoomEnvelope {
        let room_id = id.into();

        let Some(item) = self
            .client
            .sidebar
            .sidebar_items()
            .await
            .into_iter()
            .find(|item| item.room.to_generic_room().jid() == &room_id)
        else {
            panic!("Could not find connected room with id {room_id}")
        };

        item.room
    }
}

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
        let Some((expected_event, file, line)) = self.messages.pop_event() else {
            panic!("\nClient sent unexpected event:\n\n{:?}", received_event);
        };
        assert_eq!(
            expected_event, received_event,
            "\n\n➡️ Assertion failed at:\n{}:{}",
            file, line
        );
    }
}
