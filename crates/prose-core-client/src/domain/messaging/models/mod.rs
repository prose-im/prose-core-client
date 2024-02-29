// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use attachment::{Attachment, AttachmentType, Thumbnail};
pub(crate) use error::StanzaParseError;
pub use message::{Emoji, Message, Reaction};
pub use message_id::{MessageId, MessageTargetId, StanzaId};
pub use message_like::{MessageLike, MessageLikeId, Payload as MessageLikePayload};
pub use message_parser::{MessageLikeError, MessageParser};
pub use send_message_request::SendMessageRequest;

mod attachment;
mod error;
mod message;
mod message_id;
mod message_like;
mod message_parser;
mod send_message_request;
