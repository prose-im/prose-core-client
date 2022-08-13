// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::JidParseError;
use libstrophe::{ConnectClientError, ConnectionError, Stanza, ToTextError};
use std::cell::BorrowMutError;
use std::io;
use std::result::Result as StdResult;
use std::sync::mpsc::SendError;
use std::sync::PoisonError;
use strum::ParseError;
use strum_macros::Display;

#[derive(Debug, thiserror::Error, Display)]
pub enum Error {
    JidParseError { error: JidParseError },
    StropheError { error: libstrophe::Error },
    ConnectClientError { error: libstrophe::Error },
    ConnectionError { error: String },
    BorrowMutError { error: String },
    PoisonError { error: String },
    StanzaParseError { error: StanzaParseError },
    SendError { error: String },
    IOError { error: String },
    ToTextError { error: String },
    ChronoParseError { error: String },
}

#[derive(Debug, thiserror::Error, Display)]
pub enum StanzaParseError {
    MissingAttribute { attribute: String },
    MissingChildNode { node: String },
    MissingText { node: String },
    ParseError { error: String },
    JidParseError { error: JidParseError },
}

pub type Result<T, E = Error> = StdResult<T, E>;

impl From<JidParseError> for Error {
    fn from(error: JidParseError) -> Self {
        Error::JidParseError { error }
    }
}

impl From<libstrophe::Error> for Error {
    fn from(error: libstrophe::Error) -> Self {
        Error::StropheError { error }
    }
}

impl<'cb, 'cx> From<ConnectClientError<'cb, 'cx>> for Error {
    fn from(error: ConnectClientError) -> Self {
        Error::ConnectClientError { error: error.error }
    }
}

impl From<BorrowMutError> for Error {
    fn from(error: BorrowMutError) -> Self {
        Error::BorrowMutError {
            error: error.to_string(),
        }
    }
}

impl From<StanzaParseError> for Error {
    fn from(error: StanzaParseError) -> Self {
        Error::StanzaParseError { error }
    }
}

impl From<SendError<Stanza>> for Error {
    fn from(error: SendError<Stanza>) -> Self {
        Error::SendError {
            error: error.to_string(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IOError {
            error: error.to_string(),
        }
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(error: PoisonError<T>) -> Self {
        Error::PoisonError {
            error: error.to_string(),
        }
    }
}

impl<'t, 's> From<ConnectionError<'t, 's>> for Error {
    fn from(error: ConnectionError<'t, 's>) -> Self {
        Error::ConnectionError {
            error: error.to_string(),
        }
    }
}

impl From<ToTextError> for Error {
    fn from(error: ToTextError) -> Self {
        Error::ToTextError {
            error: error.to_string(),
        }
    }
}

impl From<chrono::ParseError> for Error {
    fn from(error: chrono::ParseError) -> Self {
        Error::ChronoParseError {
            error: error.to_string(),
        }
    }
}

impl StanzaParseError {
    pub fn missing_attribute(attribute: &str, _stanza: &Stanza) -> Self {
        // TODO: Derive a string for debugging from stanza
        StanzaParseError::MissingAttribute {
            attribute: attribute.to_string(),
        }
    }

    pub fn missing_child_node(node_name: &str, _stanza: &Stanza) -> Self {
        // TODO: Derive a string for debugging from stanza
        StanzaParseError::MissingChildNode {
            node: node_name.to_string(),
        }
    }

    pub fn missing_text(node_name: &str, _stanza: &Stanza) -> Self {
        // TODO: Derive a string for debugging from stanza
        StanzaParseError::MissingText {
            node: node_name.to_string(),
        }
    }
}

impl From<ParseError> for StanzaParseError {
    fn from(error: ParseError) -> Self {
        StanzaParseError::ParseError {
            error: error.to_string(),
        }
    }
}

impl From<JidParseError> for StanzaParseError {
    fn from(error: JidParseError) -> Self {
        StanzaParseError::JidParseError { error }
    }
}
