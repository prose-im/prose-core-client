use crate::cache::IndexedDBDataCacheError;
use jid::JidParseError;
use prose_xmpp::ConnectionError;
use wasm_bindgen::JsValue;

pub struct JSConnectionError(ConnectionError);

impl From<ConnectionError> for JSConnectionError {
    fn from(value: ConnectionError) -> Self {
        JSConnectionError(value)
    }
}

impl From<JidParseError> for JSConnectionError {
    fn from(_value: JidParseError) -> Self {
        JSConnectionError(ConnectionError::Generic {
            msg: "Failed to parse JID".to_string(),
        })
    }
}

impl From<JSConnectionError> for JsValue {
    fn from(value: JSConnectionError) -> Self {
        match value.0 {
            ConnectionError::TimedOut => "Connection timed out".into(),
            ConnectionError::InvalidCredentials => "Invalid credentials".into(),
            ConnectionError::Generic { msg } => msg.into(),
        }
    }
}

impl From<IndexedDBDataCacheError> for JsValue {
    fn from(value: IndexedDBDataCacheError) -> Self {
        JsValue::from_str(&value.to_string())
    }
}
