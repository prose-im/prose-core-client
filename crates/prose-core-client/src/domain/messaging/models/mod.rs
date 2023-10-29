// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub(self) use error::StanzaParseError;
pub use message::{Emoji, Message, MessageId, Reaction, StanzaId};
pub use message_like::{
    MessageLike, MessageLikeId, Payload as MessageLikePayload, TimestampedMessage,
};

mod error;
mod message;
mod message_like;
