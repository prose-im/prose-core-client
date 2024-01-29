use anyhow::{format_err, Result};
use wasm_bindgen::prelude::wasm_bindgen;

use prose_xmpp::ConnectionError as CoreConnectionError;

#[wasm_bindgen(js_name = "ProseConnectionErrorType")]
#[derive(Clone)]
pub enum ConnectionErrorType {
    TimedOut = 0,
    InvalidCredentials = 1,
    Generic = 2,
}

impl TryFrom<i32> for ConnectionErrorType {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(Self::TimedOut),
            1 => Ok(Self::InvalidCredentials),
            2 => Ok(Self::Generic),
            _ => Err(format_err!("Invalid ProseConnectionErrorType '{}'.", value)),
        }
    }
}

#[wasm_bindgen(js_name = "ProseConnectionError")]
pub struct ConnectionError {
    #[wasm_bindgen(skip)]
    pub kind: ConnectionErrorType,
    #[wasm_bindgen(skip)]
    pub message: String,
}

#[wasm_bindgen(js_class = "ProseConnectionError")]
impl ConnectionError {
    #[wasm_bindgen(getter, js_name = "type")]
    pub fn kind(&self) -> ConnectionErrorType {
        self.kind.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl From<ConnectionErrorType> for CoreConnectionError {
    fn from(value: ConnectionErrorType) -> Self {
        match value {
            ConnectionErrorType::TimedOut => CoreConnectionError::TimedOut,
            ConnectionErrorType::InvalidCredentials => CoreConnectionError::InvalidCredentials,
            ConnectionErrorType::Generic => CoreConnectionError::Generic {
                msg: "An unknown error occurred.".to_string(),
            },
        }
    }
}

impl From<CoreConnectionError> for ConnectionError {
    fn from(value: CoreConnectionError) -> Self {
        match value {
            CoreConnectionError::TimedOut => Self {
                kind: ConnectionErrorType::TimedOut,
                message: "The connection timed out.".to_string(),
            },
            CoreConnectionError::InvalidCredentials => Self {
                kind: ConnectionErrorType::InvalidCredentials,
                message: "Invalid credentials.".to_string(),
            },
            CoreConnectionError::Generic { msg } => Self {
                kind: ConnectionErrorType::Generic,
                message: msg,
            },
        }
    }
}
