// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use attachment::{Attachment, AttachmentType, Thumbnail};
pub use encrypted_message::{
    EncryptedMessage, EncryptedPayload, EncryptionKey, KeyTransportPayload,
};
pub(crate) use error::StanzaParseError;
pub use mention::Mention;
#[allow(unused_imports)] // Reaction is required in unit tests
pub use message::{Body, Emoji, Message, MessageFlags, Reaction, ReplyTo};
pub use message_id::{MessageId, MessageRemoteId, MessageServerId, MessageTargetId, ThreadId};
pub use message_like::{
    Body as MessageLikeBody, EncryptionInfo as MessageLikeEncryptionInfo, MessageLike,
    Payload as MessageLikePayload,
};
pub use message_parser::{MessageLikeError, MessageParser};
pub use message_ref::{ArchivedMessageRef, MessageRef};
pub use send_message_request::SendMessageRequest;

mod attachment;
mod encrypted_message;
mod error;
mod mention;
mod message;
mod message_id;
mod message_like;
mod message_parser;
mod message_ref;
pub mod send_message_request;
