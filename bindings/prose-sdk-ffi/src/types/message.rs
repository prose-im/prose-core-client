// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::{Attachment, Avatar};
use crate::{DateTime, Emoji, MessageId, ParticipantId, UnicodeScalarIndex, UserId};
use prose_core_client::dtos::{
    Body as CoreMessageBody, Mention as CoreMention, Message as CoreMessage,
    MessageFlags as CoreMessageFlags, MessageSender as CoreMessageSender, Reaction as CoreReaction,
    ReplyTo as CoreReplyTo, UnicodeScalarIndex as CoreUnicodeScalarIndex,
};
use std::sync::Arc;

#[derive(uniffi::Record)]
pub struct Message {
    pub id: MessageId,
    pub from: MessageSender,
    pub body: MessageBody,
    pub timestamp: DateTime,
    pub flags: MessageFlags,
    pub reactions: Vec<Reaction>,
    pub attachments: Vec<Attachment>,
    pub mentions: Vec<Mention>,
    pub reply_to: Option<ReplyTo>,
}

#[derive(uniffi::Record)]
pub struct MessageBody {
    pub raw: String,
    pub html: String,
}

#[derive(uniffi::Record)]
pub struct Mention {
    pub user: UserId,
    pub range: Option<UnicodeScalarRange>,
}

/// A (half-open) range bounded inclusively below and exclusively above (start..end).
///
/// The range start..end contains all values with start <= x < end. It is empty if start >= end.
#[derive(uniffi::Record)]
pub struct UnicodeScalarRange {
    pub start: UnicodeScalarIndex,
    pub end: UnicodeScalarIndex,
}

#[derive(uniffi::Record)]
pub struct MessageFlags {
    pub is_read: bool,
    pub is_edited: bool,
    pub is_delivered: bool,
    pub is_transient: bool,
    pub is_encrypted: bool,
    /// When contained in a list, this message is the last message that our user has read.
    pub is_last_read: bool,
}

#[derive(uniffi::Record)]
pub struct MessageSender {
    pub id: ParticipantId,
    pub name: String,
    pub avatar: Option<Arc<Avatar>>,
}

#[derive(uniffi::Record)]
pub struct Reaction {
    pub emoji: Emoji,
    pub from: Vec<MessageSender>,
}

#[derive(uniffi::Record)]
pub struct ReplyTo {
    pub id: Option<MessageId>,
    pub sender: MessageSender,
    pub timestamp: Option<DateTime>,
    pub body: Option<String>,
}

impl From<CoreMessage> for Message {
    fn from(value: CoreMessage) -> Self {
        Message {
            id: value.id.into(),
            from: value.from.into(),
            body: value.body.into(),
            timestamp: value.timestamp.into(),
            flags: value.flags.into(),
            reactions: value.reactions.into_iter().map(Into::into).collect(),
            attachments: value.attachments.into_iter().map(Into::into).collect(),
            mentions: value.mentions.into_iter().map(Into::into).collect(),
            reply_to: value.reply_to.map(Into::into),
        }
    }
}

impl From<std::ops::Range<CoreUnicodeScalarIndex>> for UnicodeScalarRange {
    fn from(value: std::ops::Range<CoreUnicodeScalarIndex>) -> Self {
        UnicodeScalarRange {
            start: value.start.into(),
            end: value.end.into(),
        }
    }
}

impl From<CoreMention> for Mention {
    fn from(mention: CoreMention) -> Self {
        Mention {
            user: mention.user.into(),
            range: mention.range.map(Into::into),
        }
    }
}

impl From<CoreMessageBody> for MessageBody {
    fn from(value: CoreMessageBody) -> Self {
        MessageBody {
            raw: value.raw,
            html: value.html.into_string(),
        }
    }
}

impl From<CoreMessageFlags> for MessageFlags {
    fn from(value: CoreMessageFlags) -> Self {
        MessageFlags {
            is_read: value.is_read,
            is_edited: value.is_edited,
            is_delivered: value.is_delivered,
            is_transient: value.is_transient,
            is_encrypted: value.is_encrypted,
            is_last_read: value.is_last_read,
        }
    }
}

impl From<CoreMessageSender> for MessageSender {
    fn from(value: CoreMessageSender) -> Self {
        MessageSender {
            id: value.id.into(),
            name: value.name,
            avatar: value.avatar.map(|a| Arc::new(a.into())),
        }
    }
}

impl From<CoreReplyTo> for ReplyTo {
    fn from(value: CoreReplyTo) -> Self {
        ReplyTo {
            id: value.id.map(Into::into),
            sender: value.sender.into(),
            timestamp: value.timestamp.map(Into::into),
            body: value.body,
        }
    }
}

impl From<CoreReaction> for Reaction {
    fn from(value: CoreReaction) -> Self {
        Reaction {
            emoji: value.emoji.into(),
            from: value.from.into_iter().map(Into::into).collect(),
        }
    }
}
