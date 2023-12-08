// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tracing::warn;
use wasm_bindgen::prelude::*;

use prose_core_client::dtos::MessageId;
use prose_core_client::{ClientDelegate, ClientEvent, ClientRoomEventType, ConnectionEvent};
use prose_xmpp::ConnectionError;

use crate::client::Client;
use crate::types::BareJid;
use crate::types::RoomEnvelopeExt;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type ConnectionTimedOutError = {
    code: 'timed_out';
}

export type ConnectionInvalidCredentialsError = {
    code: 'invalid_credentials';
}

export type ConnectionGenericError = {
    code: 'generic';
    message: string;
}

export type ConnectionError = ConnectionTimedOutError | ConnectionInvalidCredentialsError | ConnectionGenericError;

export interface ProseClientDelegate {
    clientConnected(): void
    clientDisconnected(client: ProseClient, error?: ConnectionError): void
    
    /// The contents of the sidebar have changed.
    sidebarChanged(client: ProseClient): void

    /// A user in `conversation` started or stopped typing.
    composingUsersChanged(client: ProseClient, room: Room): void

    /// Infos about a contact have changed.
    contactChanged(client: ProseClient, jid: JID): void

    /// The avatar of a user changed.
    avatarChanged(client: ProseClient, jid: JID): void

    /// One or many messages were either received or sent.
    messagesAppended(client: ProseClient, room: Room, messageIDs: string[]): void

    /// One or many messages were received that affected earlier messages (e.g. a reaction).
    messagesUpdated(client: ProseClient, room: Room, messageIDs: string[]): void

    /// A message was deleted.
    messagesDeleted(client: ProseClient, room: Room, messageIDs: string[]): void
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ProseClientDelegate")]
    pub type JSDelegate;

    #[wasm_bindgen(method, catch, js_name = "clientConnected")]
    fn client_connected(this: &JSDelegate, client: Client) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = "clientDisconnected")]
    fn client_disconnected(
        this: &JSDelegate,
        client: Client,
        error: Option<JSConnectionError>,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = "composingUsersChanged")]
    fn composing_users_changed(
        this: &JSDelegate,
        client: Client,
        room: JsValue,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = "sidebarChanged")]
    fn sidebar_changed(this: &JSDelegate, client: Client) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = "contactChanged")]
    fn contact_changed(this: &JSDelegate, client: Client, jid: BareJid) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = "avatarChanged")]
    fn avatar_changed(this: &JSDelegate, client: Client, jid: BareJid) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = "messagesAppended")]
    fn messages_appended(
        this: &JSDelegate,
        client: Client,
        room: JsValue,
        ids: Vec<JsValue>,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = "messagesUpdated")]
    fn messages_updated(
        this: &JSDelegate,
        client: Client,
        room: JsValue,
        ids: Vec<JsValue>,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = "messagesDeleted")]
    fn messages_deleted(
        this: &JSDelegate,
        client: Client,
        room: JsValue,
        ids: Vec<JsValue>,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = "roomAttributesChanged")]
    fn room_attributes_changed(
        this: &JSDelegate,
        client: Client,
        room: JsValue,
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name = "roomParticipantsChanged")]
    fn room_participants_changed(
        this: &JSDelegate,
        client: Client,
        room: JsValue,
    ) -> Result<(), JsValue>;
}

#[wasm_bindgen(getter_with_clone)]
pub struct JSConnectionError {
    pub code: String,
    pub message: Option<String>,
}

impl From<ConnectionError> for JSConnectionError {
    fn from(value: ConnectionError) -> Self {
        match value {
            ConnectionError::TimedOut => JSConnectionError {
                code: "timed_out".to_string(),
                message: None,
            },
            ConnectionError::InvalidCredentials => JSConnectionError {
                code: "invalid_credentials".to_string(),
                message: None,
            },
            ConnectionError::Generic { msg } => JSConnectionError {
                code: "generic".to_string(),
                message: Some(msg),
            },
        }
    }
}

pub struct Delegate {
    inner: JSDelegate,
}

impl Delegate {
    pub fn new(js: JSDelegate) -> Self {
        Delegate { inner: js }
    }
}

trait JSValueConvertible {
    fn into_js_array(self) -> Vec<JsValue>;
}

impl JSValueConvertible for Vec<MessageId> {
    fn into_js_array(self) -> Vec<JsValue> {
        self.into_iter()
            .map(|id| JsValue::from(id.into_inner()))
            .collect()
    }
}

impl ClientDelegate for Delegate {
    fn handle_event(&self, client: prose_core_client::Client, event: ClientEvent) {
        match self.handle_event_throwing(client, event) {
            Ok(()) => (),
            Err(val) => warn!(
                "JSDelegate threw an error when handling an event: {:?}",
                val
            ),
        }
    }
}

impl Delegate {
    fn handle_event_throwing(
        &self,
        client: prose_core_client::Client,
        event: ClientEvent,
    ) -> Result<(), JsValue> {
        let client = Client::from(client);

        match event {
            ClientEvent::ConnectionStatusChanged {
                event: ConnectionEvent::Connect,
            } => self.inner.client_connected(client)?,
            ClientEvent::ConnectionStatusChanged {
                event: ConnectionEvent::Disconnect { error },
            } => self
                .inner
                .client_disconnected(client, error.map(Into::into))?,
            ClientEvent::SidebarChanged => self.inner.sidebar_changed(client)?,
            ClientEvent::ContactChanged { id: jid } => self
                .inner
                .contact_changed(client, jid.into_inner().into())?,
            ClientEvent::AvatarChanged { id: jid } => {
                self.inner.avatar_changed(client, jid.into_inner().into())?
            }
            ClientEvent::RoomChanged { room, r#type } => match r#type {
                ClientRoomEventType::MessagesAppended { message_ids } => self
                    .inner
                    .messages_appended(client, room.into_js_value(), message_ids.into_js_array())?,
                ClientRoomEventType::MessagesUpdated { message_ids } => self
                    .inner
                    .messages_updated(client, room.into_js_value(), message_ids.into_js_array())?,
                ClientRoomEventType::MessagesDeleted { message_ids } => self
                    .inner
                    .messages_deleted(client, room.into_js_value(), message_ids.into_js_array())?,
                ClientRoomEventType::ComposingUsersChanged => self
                    .inner
                    .composing_users_changed(client, room.into_js_value())?,
                ClientRoomEventType::AttributesChanged => self
                    .inner
                    .room_attributes_changed(client, room.into_js_value())?,
                ClientRoomEventType::ParticipantsChanged => self
                    .inner
                    .room_participants_changed(client, room.into_js_value())?,
            },
        }
        Ok(())
    }
}
