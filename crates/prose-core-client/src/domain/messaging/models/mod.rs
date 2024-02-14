// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use attachment::{Attachment, AttachmentType, Thumbnail};
pub(crate) use error::StanzaParseError;
pub use message::{Emoji, Message, MessageId, Reaction, StanzaId};
pub use message_like::{
    MessageLike, MessageLikeError, MessageLikeId, Payload as MessageLikePayload, TimestampedMessage,
};
pub use send_message_request::SendMessageRequest;

mod attachment;
mod error;
mod message;
mod message_like;
mod send_message_request;
