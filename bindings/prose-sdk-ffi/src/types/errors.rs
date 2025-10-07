// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_core_client::FsAvatarRepositoryError;

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(uniffi::Error, thiserror::Error, Debug)]
pub enum ConnectionError {
    #[error("Timed out")]
    TimedOut,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("{msg:?}")]
    Generic { msg: String },
}

#[derive(uniffi::Error, thiserror::Error, Debug)]
pub enum ClientError {
    #[error("client error: {msg}")]
    Generic { msg: String },
}

#[derive(uniffi::Error, thiserror::Error, Debug)]
pub enum JidParseError {
    /// Happens when the node is empty, that is the string starts with a @.
    #[error("Node is empty (string starts with @)")]
    NodeEmpty,

    /// Happens when there is no domain, that is either the string is empty,
    /// starts with a /, or contains the @/ sequence.
    #[error("Domain is empty")]
    DomainEmpty,

    /// Happens when the resource is empty, that is the string ends with a /.
    #[error("Resource is empty (string ends with /)")]
    ResourceEmpty,

    /// Happens when the localpart is longer than 1023 bytes.
    #[error("Node is too long (max 1023 bytes)")]
    NodeTooLong,

    /// Happens when the domain is longer than 1023 bytes.
    #[error("Domain is too long (max 1023 bytes)")]
    DomainTooLong,

    /// Happens when the resource is longer than 1023 bytes.
    #[error("Resource is too long (max 1023 bytes)")]
    ResourceTooLong,

    /// Happens when the localpart is invalid according to nodeprep.
    #[error("Invalid node (failed nodeprep validation)")]
    NodePrep,

    /// Happens when the domain is invalid according to nameprep.
    #[error("Invalid domain (failed nameprep validation)")]
    NamePrep,

    /// Happens when the resource is invalid according to resourceprep.
    #[error("Invalid resource (failed resourceprep validation)")]
    ResourcePrep,

    /// Happens when there is no resource, that is string contains no /.
    #[error("Resource is missing in full JID")]
    ResourceMissingInFullJid,

    /// Happens when parsing a bare JID and there is a resource.
    #[error("Resource found in bare JID")]
    ResourceInBareJid,
}

impl From<prose_xmpp::ConnectionError> for ConnectionError {
    fn from(err: prose_xmpp::ConnectionError) -> Self {
        match err {
            prose_xmpp::ConnectionError::TimedOut => Self::TimedOut,
            prose_xmpp::ConnectionError::InvalidCredentials => Self::InvalidCredentials,
            prose_xmpp::ConnectionError::Generic { msg } => Self::Generic { msg },
        }
    }
}

impl From<anyhow::Error> for ClientError {
    fn from(e: anyhow::Error) -> ClientError {
        ClientError::Generic { msg: e.to_string() }
    }
}

impl From<FsAvatarRepositoryError> for ClientError {
    fn from(e: FsAvatarRepositoryError) -> Self {
        ClientError::Generic { msg: e.to_string() }
    }
}

impl From<jid::Error> for JidParseError {
    fn from(e: jid::Error) -> Self {
        match e {
            jid::Error::NodeEmpty => Self::NodeEmpty,
            jid::Error::DomainEmpty => Self::DomainEmpty,
            jid::Error::ResourceEmpty => Self::ResourceEmpty,
            jid::Error::NodeTooLong => Self::NodeTooLong,
            jid::Error::DomainTooLong => Self::DomainTooLong,
            jid::Error::ResourceTooLong => Self::ResourceTooLong,
            jid::Error::NodePrep => Self::NodePrep,
            jid::Error::NamePrep => Self::NamePrep,
            jid::Error::ResourcePrep => Self::ResourcePrep,
            jid::Error::ResourceMissingInFullJid => Self::ResourceMissingInFullJid,
            jid::Error::ResourceInBareJid => Self::ResourceInBareJid,
        }
    }
}
