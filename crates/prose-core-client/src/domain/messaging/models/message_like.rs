// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Debug;

use chrono::{DateTime, Utc};
use jid::BareJid;
use serde::{Deserialize, Serialize};

use prose_xmpp::stanza::message;

use crate::domain::messaging::models::message_id::MessageId;
use crate::domain::messaging::models::{Attachment, Mention, MessageTargetId};
use crate::domain::shared::models::{ParticipantId, HTML};
use crate::dtos::DeviceId;

use super::{MessageRemoteId, MessageServerId};

/// A type that describes permanent messages, i.e. messages that need to be replayed to restore
/// the complete history of a conversation. Note that ephemeral messages like chat states are
/// handled differently.
#[derive(Debug, PartialEq, Clone)]
pub struct MessageLike {
    pub id: MessageId,
    pub remote_id: Option<MessageRemoteId>,
    pub server_id: Option<MessageServerId>,
    pub target: Option<MessageTargetId>,
    pub to: Option<BareJid>,
    pub from: ParticipantId,
    pub timestamp: DateTime<Utc>,
    pub payload: Payload,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct EncryptionInfo {
    pub sender: DeviceId,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Body {
    /// Contains Markdown text if the message was sent by a Prose client otherwise the raw body
    /// text, which may or may not be formatted according to XEP-0393: Message Styling.
    pub raw: String,

    /// Contains either the Markdown text converted to HTML, or if the message did not include
    /// Markdown the fallback message wrapped in an HTML paragraph.
    pub html: HTML,

    /// Contains any mentions that are contained in the message.
    pub mentions: Vec<Mention>,
}

impl Default for Body {
    fn default() -> Self {
        Self {
            raw: String::new(),
            html: HTML::new(""),
            mentions: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum Payload {
    Error {
        message: String,
    },
    Correction {
        body: Body,
        attachments: Vec<Attachment>,
        // Set if the message was encrypted
        encryption_info: Option<EncryptionInfo>,
    },
    DeliveryReceipt,
    ReadReceipt,
    Message {
        body: Body,
        attachments: Vec<Attachment>,
        // Set if the message was encrypted
        encryption_info: Option<EncryptionInfo>,
        is_transient: bool,
    },
    Reaction {
        emojis: Vec<message::Emoji>,
    },
    Retraction,
}

impl Payload {
    pub fn is_message(&self) -> bool {
        match self {
            Self::Message { .. } => true,
            _ => false,
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            Self::Error { .. } => true,
            _ => false,
        }
    }
}
