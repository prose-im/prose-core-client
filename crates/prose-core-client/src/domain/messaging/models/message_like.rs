// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Display};

use anyhow::Result;
use chrono::{DateTime, Utc};
use jid::BareJid;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use prose_xmpp::stanza::message;

use crate::domain::messaging::models::{Attachment, MessageTargetId};
use crate::domain::shared::models::ParticipantId;

use super::{MessageId, StanzaId};

/// An ID that can act as a placeholder in the rare cases when a message doesn't have an ID. Since
/// our DataCache backends require some ID for each message we simply generate one.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MessageLikeId(MessageId);

/// A type that describes permanent messages, i.e. messages that need to be replayed to restore
/// the complete history of a conversation. Note that ephemeral messages like chat states are
/// handled differently.
#[derive(Debug, PartialEq, Clone)]
pub struct MessageLike {
    pub id: MessageLikeId,
    pub stanza_id: Option<StanzaId>,
    pub target: Option<MessageTargetId>,
    pub to: Option<BareJid>,
    pub from: ParticipantId,
    pub timestamp: DateTime<Utc>,
    pub payload: Payload,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum Payload {
    Correction {
        body: String,
        attachments: Vec<Attachment>,
    },
    DeliveryReceipt,
    ReadReceipt,
    Message {
        body: String,
        attachments: Vec<Attachment>,
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
}

impl MessageLikeId {
    pub fn new(id: Option<MessageId>) -> Self {
        if let Some(id) = id {
            return MessageLikeId(id);
        }
        return MessageLikeId(format!("!!{}", Uuid::new_v4().to_string()).into());
    }

    /// Returns either the original message ID or the generated one.
    pub fn id(&self) -> &MessageId {
        &self.0
    }

    /// Returns the original message ID or None if we contain a generated ID.
    pub fn into_original_id(self) -> Option<MessageId> {
        if self.0.as_ref().starts_with("!!") {
            return None;
        }
        return Some(self.0);
    }

    pub fn original_id(&self) -> Option<&MessageId> {
        if self.0.as_ref().starts_with("!!") {
            return None;
        }
        return Some(&self.0);
    }
}

impl std::str::FromStr for MessageLikeId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(MessageLikeId(s.to_string().into()))
    }
}

impl Display for MessageLikeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
