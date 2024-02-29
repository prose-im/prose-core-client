// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Display};

use anyhow::Result;
use chrono::{DateTime, Utc};
use jid::BareJid;
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;
use xmpp_parsers::message::MessageType;

use prose_xmpp::mods::chat::Carbon;
use prose_xmpp::stanza::message;
use prose_xmpp::stanza::message::{mam, stanza_id, Forwarded, Message};

use crate::domain::messaging::models::{Attachment, MessageTargetId};
use crate::domain::shared::models::ParticipantId;
use crate::infra::xmpp::type_conversions::stanza_error::StanzaErrorExt;
use crate::infra::xmpp::util::MessageExt;

use super::{MessageId, StanzaId, StanzaParseError};

#[derive(thiserror::Error, Debug)]
pub enum MessageLikeError {
    #[error("No payload in message")]
    NoPayload,
}

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

/// An ID that can act as a placeholder in the rare cases when a message doesn't have an ID. Since
/// our DataCache backends require some ID for each message we simply generate one.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MessageLikeId(MessageId);

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

/// A wrapper for messages that might not contain a `delay` node with a timestamp, i.e. a received
/// or sent message (or more generally: a message not loaded from MAM).
pub struct TimestampedMessage<T> {
    pub message: T,
    pub timestamp: DateTime<Utc>,
}

impl TryFrom<TimestampedMessage<Carbon>> for MessageLike {
    type Error = anyhow::Error;

    fn try_from(envelope: TimestampedMessage<Carbon>) -> Result<Self> {
        let carbon = match envelope.message {
            Carbon::Received(carbon) => carbon,
            Carbon::Sent(carbon) => carbon,
        };

        let stanza_id = carbon
            .stanza
            .as_ref()
            .and_then(|s| s.stanza_id())
            .map(|sid| sid.id);
        MessageLike::try_from((stanza_id, &carbon))
    }
}

impl TryFrom<TimestampedMessage<Message>> for MessageLike {
    type Error = anyhow::Error;

    fn try_from(envelope: TimestampedMessage<Message>) -> Result<Self> {
        let msg = envelope.message;

        let id = MessageLikeId::new(msg.id.as_ref().map(|id| id.into()));
        let stanza_id = msg.stanza_id();
        let from = msg.resolved_from()?;
        let to = msg.to.as_ref();
        let timestamp = msg
            .delay()
            .map(|delay| delay.stamp.0.into())
            .unwrap_or(envelope.timestamp);
        let TargetedPayload { target, payload } = TargetedPayload::try_from(&msg)?;

        Ok(MessageLike {
            id,
            stanza_id: stanza_id.map(|s| s.id.as_ref().into()),
            target,
            to: to.map(|jid| jid.to_bare()),
            from,
            timestamp: timestamp.into(),
            payload,
        })
    }
}

impl TryFrom<&mam::ArchivedMessage> for MessageLike {
    type Error = anyhow::Error;

    fn try_from(carbon: &mam::ArchivedMessage) -> Result<Self> {
        MessageLike::try_from((Some(carbon.id.clone()), &carbon.forwarded))
    }
}

impl TryFrom<(Option<stanza_id::Id>, &Forwarded)> for MessageLike {
    type Error = anyhow::Error;

    fn try_from(value: (Option<stanza_id::Id>, &Forwarded)) -> Result<Self> {
        let Some(stanza_id) = value.0 else {
            return Err(anyhow::format_err!("Missing stanza_id in ForwardedMessage"));
        };
        let carbon = value.1;

        let message = *carbon
            .stanza
            .as_ref()
            .ok_or(StanzaParseError::missing_child_node("message"))?
            .clone();

        let TargetedPayload { target, payload } = TargetedPayload::try_from(&message)?;

        let id = MessageLikeId::new(message.id.as_ref().map(|id| id.into()));
        let to = message.to.as_ref();
        let from = message.resolved_from()?;
        let timestamp = &carbon
            .delay
            .as_ref()
            .ok_or(StanzaParseError::missing_child_node("delay"))?
            .stamp;

        Ok(MessageLike {
            id,
            stanza_id: Some(stanza_id.as_ref().into()),
            target,
            to: to.map(|jid| jid.to_bare()),
            from,
            timestamp: timestamp.0.into(),
            payload,
        })
    }
}

struct TargetedPayload {
    target: Option<MessageTargetId>,
    payload: Payload,
}

impl TryFrom<&Message> for TargetedPayload {
    type Error = anyhow::Error;

    fn try_from(message: &Message) -> Result<Self> {
        if let Some(error) = &message.error() {
            return Ok(TargetedPayload {
                target: None,
                payload: Payload::Message {
                    body: format!("Error: {}", error.to_string()),
                    attachments: vec![],
                },
            });
        }

        if let Some(reactions) = message.reactions() {
            return Ok(TargetedPayload {
                target: Some(match message.type_ {
                    MessageType::Groupchat => MessageTargetId::StanzaId(reactions.id.into()),
                    _ => MessageTargetId::MessageId(reactions.id.into()),
                }),
                payload: Payload::Reaction {
                    emojis: reactions.reactions,
                },
            });
        };

        if let Some(fastening) = message.fastening() {
            if fastening.retract() {
                return Ok(TargetedPayload {
                    target: Some(MessageTargetId::MessageId(fastening.id.as_ref().into())),
                    payload: Payload::Retraction,
                });
            }
        }

        if let (Some(replace_id), Some(body)) = (message.replace(), message.body()) {
            return Ok(TargetedPayload {
                target: Some(MessageTargetId::MessageId(replace_id.as_ref().into())),
                payload: Payload::Correction {
                    body: body.to_string(),
                    attachments: message.attachments(),
                },
            });
        }

        if let Some(marker) = message.received_marker() {
            return Ok(TargetedPayload {
                target: Some(MessageTargetId::MessageId(marker.id.as_ref().into())),
                payload: Payload::DeliveryReceipt,
            });
        }

        if let Some(marker) = message.displayed_marker() {
            return Ok(TargetedPayload {
                target: Some(MessageTargetId::MessageId(marker.id.as_ref().into())),
                payload: Payload::ReadReceipt,
            });
        }

        if let Some(body) = message.body() {
            return Ok(TargetedPayload {
                target: None,
                payload: Payload::Message {
                    body: body.to_string(),
                    attachments: message.attachments(),
                },
            });
        }

        error!("Failed to parse message {:?}", message);
        Err(MessageLikeError::NoPayload.into())
    }
}
