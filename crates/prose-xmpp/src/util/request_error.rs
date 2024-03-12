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
    #[error(transparent)]
    JidError(#[from] jid::Error),
    #[error("Request error: {msg}")]
    Generic { msg: String },
    #[error(transparent)]
    ParseError(#[from] ParseError),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Parse error: {msg}")]
    Generic { msg: String },
    #[error(transparent)]
    XMPPParseError(#[from] xmpp_parsers::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl From<xmpp_parsers::Error> for RequestError {
    fn from(value: xmpp_parsers::Error) -> Self {
        Self::ParseError(value.into())
    }
}

impl From<StanzaError> for RequestError {
    fn from(value: StanzaError) -> Self {
        Self::XMPP { err: value }
    }
}

impl RequestError {
    pub fn is_item_not_found_err(&self) -> bool {
        self.defined_condition() == Some(DefinedCondition::ItemNotFound)
    }

    pub fn is_forbidden_err(&self) -> bool {
        self.defined_condition() == Some(DefinedCondition::Forbidden)
    }

    pub fn defined_condition(&self) -> Option<DefinedCondition> {
        let RequestError::XMPP {
            err: StanzaError {
                defined_condition, ..
            },
        } = self
        else {
            return None;
        };
        return Some(defined_condition.clone());
    }
}
