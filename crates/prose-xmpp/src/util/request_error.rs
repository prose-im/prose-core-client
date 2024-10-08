// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::stanza_error::{DefinedCondition, StanzaError};
use xso::error::FromElementError;

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("Request Timeout")]
    TimedOut,
    #[error("Client is disconnected")]
    Disconnected,
    #[error("Request Error: Unexpected server response")]
    UnexpectedResponse,
    #[error("XMPP Error: {err:?}")]
    XMPP { err: StanzaError },
    #[error("Request error: {msg}")]
    Generic { msg: String },
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    FromElementError(#[from] FromElementError),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Parse error: {msg}")]
    Generic { msg: String },
    #[error(transparent)]
    XMPPParseError(#[from] xmpp_parsers::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    JidError(#[from] jid::Error),
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

    pub fn is_feature_not_implemented_err(&self) -> bool {
        self.defined_condition() == Some(DefinedCondition::FeatureNotImplemented)
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
