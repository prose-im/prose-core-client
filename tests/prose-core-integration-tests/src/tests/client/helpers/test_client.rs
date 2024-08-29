// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp::max;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use minidom::Element;
use parking_lot::Mutex;
use pretty_assertions::assert_eq;
use regex::Regex;

use prose_core_client::app::deps::AppConfig;
use prose_core_client::domain::encryption::services::IncrementingUserDeviceIdProvider;
use prose_core_client::domain::messaging::services::WrappingMessageIdProvider;
use prose_core_client::dtos::{DeviceId, MessageId, RoomEnvelope, RoomId, UserId};
use prose_core_client::infra::encryption::{EncryptionKeysRepository, SessionRepository};
use prose_core_client::infra::general::mocks::StepRngProvider;
use prose_core_client::test::ConstantTimeProvider;
use prose_core_client::{
    Client, ClientEvent, ClientRoomEventType, FsAvatarRepository, SignalServiceHandle,
};
use prose_store::prelude::Store;
use prose_xmpp::stanza::VCard4;
use prose_xmpp::test::IncrementingIDProvider;

use crate::tests::client::helpers::delegate::Delegate;
use crate::tests::client::helpers::id_provider::IncrementingOffsettingIDProvider;
use crate::tests::{store, PlatformDriver};

use super::{connector::Connector, test_message_queue::TestMessageQueue};

#[allow(dead_code)]
pub struct TestClient {
    pub(super) client: Client,
    connector: Connector,
    pub(crate) id_provider: IncrementingOffsettingIDProvider,
    pub(crate) message_id_provider: IncrementingOffsettingIDProvider,
    pub(super) short_id_provider: IncrementingIDProvider,
    messages: TestMessageQueue,
    context: Mutex<Vec<HashMap<String, String>>>,
    pub time_provider: ConstantTimeProvider,
    pub app_config: AppConfig,
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
            "TestClient dropped while still containing {num_remaining_messages} messages.\n\n{:?}",
            self.messages
        );
    }
}

pub struct TestClientBuilder {
    time_provider: ConstantTimeProvider,
    store: Option<Store<PlatformDriver>>,
    app_config: AppConfig,
}

impl TestClientBuilder {
    fn new() -> Self {
        Self {
            time_provider: ConstantTimeProvider::ymd(2024, 02, 19),
            store: None,
            app_config: Default::default(),
        }
    }
}

impl TestClientBuilder {
    pub fn set_time_provider(self, time_provider: ConstantTimeProvider) -> Self {
        Self {
            time_provider,
            store: self.store,
            app_config: self.app_config,
        }
    }

    pub fn set_store(self, store: Store<PlatformDriver>) -> Self {
        Self {
            time_provider: self.time_provider,
            store: Some(store),
            app_config: self.app_config,
        }
    }
}

impl TestClientBuilder {
    pub async fn build(self) -> TestClient {
        let messages = TestMessageQueue::default();
        let connector = Connector::new(messages.clone());
        let path = tempfile::tempdir().unwrap().path().join("avatars");
        let device_id = DeviceId::from(TestClient::device_id());
        let store = if let Some(store) = self.store {
            store
        } else {
            store().await.expect("Failed to set up store.")
        };

        let encryption_service = SignalServiceHandle::new(
            Arc::new(EncryptionKeysRepository::new(store.clone())),
            Arc::new(SessionRepository::new(store.clone())),
            Arc::new(StepRngProvider::default()),
        );

        let client = Client::builder()
            .set_connector_provider(connector.provider())
            .set_id_provider(IncrementingIDProvider::new("id"))
            .set_message_id_provider(WrappingMessageIdProvider::incrementing("msg-id"))
            .set_short_id_provider(IncrementingIDProvider::new("short-id"))
            .set_rng_provider(StepRngProvider::default())
            .set_avatar_repository(FsAvatarRepository::new(&path).unwrap())
            .set_encryption_service(Arc::new(encryption_service))
            .set_store(store)
            .set_time_provider(self.time_provider.clone())
            .set_user_device_id_provider(IncrementingUserDeviceIdProvider::new(*device_id.as_ref()))
            .set_delegate(Some(Box::new(Delegate::new(messages.clone()))))
            .set_config(self.app_config.clone())
            .build();

        let client = TestClient {
            client,
            // We'll just mirror the used ID provider here…
            connector,
            id_provider: IncrementingOffsettingIDProvider::new("id"),
            message_id_provider: IncrementingOffsettingIDProvider::new("msg-id"),
            short_id_provider: IncrementingIDProvider::new("short-id"),
            messages,
            context: Default::default(),
            time_provider: self.time_provider,
            app_config: self.app_config,
        };

        client.push_ctx([("USER_DEVICE_ID", format!("{}", device_id.as_ref()))]);

        client
    }
}

impl TestClient {
    pub async fn new() -> Self {
        Self::builder().build().await
    }

    pub fn builder() -> TestClientBuilder {
        TestClientBuilder::new()
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

#[macro_export]
macro_rules! room_event(
    ($client:ident, $room_id:expr, $event_type:expr) => (
        $client.room_event($room_id, $event_type, file!(), line!())
    )
);

#[macro_export]
macro_rules! any_event(
    ($client:ident) => (
        $client.any_event(file!(), line!())
    )
);

#[allow(dead_code)]
impl TestClient {
    pub fn send(&self, xml: impl Into<String>, file: &str, line: u32) {
        self.id_provider.apply_offset();
        self.message_id_provider.apply_offset();

        let mut xml = xml.into();

        let regex = Regex::new(r"\{\{ID(?::([0-9]+))?}}").unwrap();
        let mut highest_offset = 0i64;

        let unmodified_xml = xml.clone();
        let captures = regex.captures_iter(&unmodified_xml).collect::<Vec<_>>();

        for c in captures.into_iter().rev() {
            let full_capture = c.get(0).unwrap();

            let offset = c
                .get(1)
                .map(|s| s.as_str().parse::<i64>().unwrap())
                .unwrap_or(1);
            assert!(offset >= 1, "ID offsets must start at 1");
            highest_offset = max(highest_offset, offset);

            xml.replace_range(
                full_capture.start()..full_capture.end(),
                &self.id_provider.id_with_offset(offset),
            );
        }

        self.id_provider.set_offset(highest_offset);

        self.apply_ctx(&mut xml);

        if xml.contains("{{MSG_ID}}") {
            self.push_ctx([("MSG_ID", self.message_id_provider.id_with_offset(1))]);
            self.apply_ctx(&mut xml);
            self.pop_ctx();
            self.message_id_provider.set_offset(1);
        };

        self.messages.send(xml, file, line);
    }

    pub fn receive(&self, xml: impl Into<String>, file: &str, line: u32) {
        let mut xml = xml.into();

        let regex = Regex::new(r"\{\{ID(?::([0-9]+))?}}").unwrap();

        let unmodified_xml = xml.clone();
        let captures = regex.captures_iter(&unmodified_xml).collect::<Vec<_>>();

        for c in captures.into_iter().rev() {
            let full_capture = c.get(0).unwrap();

            let offset = c
                .get(1)
                .map(|s| s.as_str().parse::<i64>().unwrap())
                .unwrap_or(1);
            assert!(offset >= 1, "ID offsets must start at 1");

            xml.replace_range(
                full_capture.start()..full_capture.end(),
                &self.id_provider.last_id_with_offset(offset),
            );
        }

        self.apply_ctx(&mut xml);

        if xml.contains("{{LAST_MSG_ID}}") {
            self.push_ctx([(
                "LAST_MSG_ID",
                self.message_id_provider.last_id_with_offset(1),
            )]);
            self.apply_ctx(&mut xml);
            self.pop_ctx();
        };

        if xml.contains("{{MSG_ID}}") {
            self.push_ctx([("MSG_ID", self.message_id_provider.id_with_offset(1))]);
            self.apply_ctx(&mut xml);
            self.pop_ctx();
            self.message_id_provider.set_offset(1);
        };

        self.messages.receive(xml, file, line);
    }

    pub fn receive_element(&self, element: impl Into<Element>, file: &str, line: u32) {
        self.messages.receive_element(element, file, line);
    }

    pub fn event(&self, event: ClientEvent, file: &str, line: u32) {
        self.messages.event(event, file, line);
    }

    pub fn room_event(
        &self,
        room_id: impl Into<RoomId>,
        event_type: ClientRoomEventType,
        file: &str,
        line: u32,
    ) {
        self.messages
            .room_event(room_id.into(), event_type, file, line);
    }

    pub fn any_event(&self, file: &str, line: u32) {
        self.messages.any_event(file, line)
    }

    pub async fn receive_next(&self) {
        self.connector.receive_next().await
    }

    pub async fn simulate_disconnect(&self) {
        self.connector.send_disconnect().await
    }

    pub async fn simulate_ping_timer(&self) {
        self.connector.send_ping_timer_event().await
    }

    pub async fn simulate_timeout_timer(&self) {
        self.connector.send_timeout_timer_event().await
    }
}

impl TestClient {
    pub fn push_ctx<T, U>(&self, ctx: impl IntoIterator<Item = (T, U)>)
    where
        T: Into<String>,
        U: Into<String>,
    {
        self.context
            .lock()
            .push(ctx.into_iter().map(|(k, v)| (k.into(), v.into())).collect());
    }

    pub fn pop_ctx(&self) {
        self.context.lock().pop();
    }

    pub fn get_ctx(&self, key: impl AsRef<str>) -> Option<String> {
        self.context
            .lock()
            .iter()
            .find_map(|ctx| ctx.get(key.as_ref()).cloned())
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

#[allow(dead_code)]
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

    pub fn get_last_id(&self) -> String {
        self.id_provider.last_id_with_offset(1)
    }

    pub fn get_next_id(&self) -> String {
        self.id_provider.next_id()
    }

    pub fn get_last_message_id(&self) -> MessageId {
        self.message_id_provider.last_id_with_offset(1).into()
    }

    pub fn get_next_message_id_with_offset(&self, offset: i64) -> MessageId {
        assert!(offset > 0);
        self.message_id_provider.id_with_offset(offset).into()
    }

    pub fn get_next_message_id(&self) -> MessageId {
        self.message_id_provider.next_id().into()
    }

    pub fn bump_message_id(&self) {
        self.message_id_provider.set_offset(1);
        self.message_id_provider.apply_offset();
    }
}

impl TestClient {
    pub fn expect_load_vcard(&self, user_id: &UserId) {
        self.push_ctx([("OTHER_USER_ID", user_id.to_string())]);
        send!(
            self,
            r#"
            <iq xmlns="jabber:client" id="{{ID}}" to="{{OTHER_USER_ID}}" type="get">
              <vcard xmlns="urn:ietf:params:xml:ns:vcard-4.0" />
            </iq>
            "#
        );
        self.pop_ctx();
    }

    pub fn receive_vcard(&self, user_id: &UserId, vcard4: VCard4) {
        self.push_ctx([
            ("SENDER_ID", user_id.to_string()),
            ("VCARD", String::from(&Element::from(vcard4))),
        ]);
        recv!(
            self,
            r#"
            <message xmlns="jabber:client" from="{{SENDER_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="headline">
              <event xmlns="http://jabber.org/protocol/pubsub#event">
                <items node="urn:ietf:params:xml:ns:vcard-4.0">
                  <item id="{{SENDER_ID}}" publisher="{{SENDER_ID}}">
                    {{VCARD}}
                  </item>
                </items>
              </event>
            </message>
            "#
        );
        self.pop_ctx();
    }

    pub fn receive_nickname(&self, user_id: &UserId, nickname: impl AsRef<str>) {
        self.push_ctx([
            ("SENDER_ID", user_id.to_string()),
            ("NICKNAME", nickname.as_ref().to_string()),
        ]);
        recv!(
            self,
            r#"
            <message xmlns="jabber:client" from="{{SENDER_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="headline">
              <event xmlns="http://jabber.org/protocol/pubsub#event">
                <items node="http://jabber.org/protocol/nick">
                  <item>
                    <nick xmlns="http://jabber.org/protocol/nick">{{NICKNAME}}</nick>
                  </item>
                </items>
              </event>
            </message>
            "#
        );
        self.pop_ctx();
    }

    pub fn expect_load_avatar_metadata(&self, user_id: &UserId) {
        self.push_ctx([("OTHER_USER_ID", user_id.to_string())]);
        send!(
            self,
            r#"
            <iq xmlns="jabber:client" id="{{ID}}" to="{{OTHER_USER_ID}}" type="get">
              <pubsub xmlns="http://jabber.org/protocol/pubsub">
                <items max_items="1" node="urn:xmpp:avatar:metadata" />
              </pubsub>
            </iq>
            "#
        );
        self.pop_ctx();
    }

    pub fn receive_not_found_iq_response(&self) {
        recv!(
            self,
            r#"
            <iq xmlns="jabber:client" id="{{ID}}" type="error">
              <error type="cancel">
                <item-not-found xmlns="urn:ietf:params:xml:ns:xmpp-stanzas" />
              </error>
            </iq>
            "#
        );
    }
}

impl Deref for TestClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
