// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::stanza_error::{DefinedCondition, StanzaError};

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("Request Timeout")]
    TimedOut,
    #[error("Request Error: Unexpected server response")]
    UnexpectedResponse,
    #[error("XMPP Error: {err:?}")]
    XMPP { err: StanzaError },
    #[error("Request error: {msg}")]
    Generic { msg: String },
}

impl RequestError {
    pub fn is_item_not_found_err(&self) -> bool {
        if let RequestError::XMPP {
            err:
                StanzaError {
                    defined_condition: DefinedCondition::ItemNotFound,
                    ..
                },
        } = self
        {
            return true;
        }
        false
    }
}
