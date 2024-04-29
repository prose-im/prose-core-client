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
pub use message::{Emoji, Message, Reaction};
pub use message_id::{MessageId, MessageTargetId, StanzaId};
pub use message_like::{
    EncryptionInfo as MessageLikeEncryptionInfo, MessageLike, MessageLikeId,
    Payload as MessageLikePayload,
};
pub use message_parser::{MessageLikeError, MessageParser};
pub use send_message_request::SendMessageRequest;

mod attachment;
mod encrypted_message;
mod error;
mod mention;
mod message;
mod message_id;
mod message_like;
mod message_parser;
pub mod send_message_request;
