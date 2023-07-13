use wasm_bindgen::prelude::*;

use prose_core_client::{ClientDelegate, ClientEvent, ConnectionEvent};
use prose_domain::MessageId;
use prose_xmpp::ConnectionError;

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
    clientConnected()
    clientDisconnected(error?: ConnectionError)

    /// A user in `conversation` started or stopped typing.
    composingUsersChanged(conversation: string)
    
    /// Infos about a contact have changed.
    contactChanged(jid: string)
    
    /// One or many messages were either received or sent.
    messagesAppended(conversation: string, messageIDs: string[])

    /// One or many messages were received that affected earlier messages (e.g. a reaction).
    messagesUpdated(conversation: string, messageIDs: string[])
    
    /// A message was deleted.
    messagesDeleted(conversation: string, messageIDs: string[])
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ProseClientDelegate")]
    pub type JSDelegate;

    #[wasm_bindgen(method, js_name = "clientConnected")]
    fn client_connected(this: &JSDelegate);

    #[wasm_bindgen(method, js_name = "clientDisconnected")]
    fn client_disconnected(this: &JSDelegate, error: Option<JSConnectionError>);

    #[wasm_bindgen(method, js_name = "composingUsersChanged")]
    fn composing_users_changed(this: &JSDelegate, conversation: String);

    #[wasm_bindgen(method, js_name = "contactChanged")]
    fn contactChanged(this: &JSDelegate, jid: String);

    #[wasm_bindgen(method, js_name = "messagesAppended")]
    fn messages_appended(this: &JSDelegate, conversation: String, ids: Vec<JsValue>);

    #[wasm_bindgen(method, js_name = "messagesUpdated")]
    fn messages_updated(this: &JSDelegate, conversation: String, ids: Vec<JsValue>);

    #[wasm_bindgen(method, js_name = "messagesDeleted")]
    fn messages_deleted(this: &JSDelegate, conversation: String, ids: Vec<JsValue>);
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
        self.into_iter().map(|id| JsValue::from(id.0)).collect()
    }
}

impl ClientDelegate for Delegate {
    fn handle_event(&self, event: ClientEvent) {
        match event {
            ClientEvent::ComposingUsersChanged { conversation } => {
                self.inner.composing_users_changed(conversation.to_string())
            }
            ClientEvent::ConnectionStatusChanged {
                event: ConnectionEvent::Connect,
            } => self.inner.client_connected(),
            ClientEvent::ConnectionStatusChanged {
                event: ConnectionEvent::Disconnect { error },
            } => self.inner.client_disconnected(error.map(Into::into)),
            ClientEvent::ContactChanged { jid } => self.inner.contactChanged(jid.to_string()),
            ClientEvent::MessagesAppended {
                conversation,
                message_ids,
            } => self
                .inner
                .messages_appended(conversation.to_string(), message_ids.into_js_array()),
            ClientEvent::MessagesUpdated {
                conversation,
                message_ids,
            } => self
                .inner
                .messages_updated(conversation.to_string(), message_ids.into_js_array()),
            ClientEvent::MessagesDeleted {
                conversation,
                message_ids,
            } => self
                .inner
                .messages_deleted(conversation.to_string(), message_ids.into_js_array()),
        }
    }
}
