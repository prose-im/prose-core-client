use chrono::{DateTime, Utc};
use itertools::Itertools;
use jid::BareJid;
use serde::{Deserialize, Serialize};

use prose_core_lib::modules::{ArchivedMessage, ReceivedMessage};
use prose_core_lib::stanza::message::{Emoji, StanzaId};
use prose_core_lib::stanza::{message, ForwardedMessage, Message, StanzaBase};

use crate::types::error::StanzaParseError;

/// A type that describes permanent messages, i.e. messages that need to be replayed to restore
/// the complete history of a conversation. Note that ephemeral messages like chat states are
/// handled differently.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MessageLike {
    pub id: message::Id,
    pub stanza_id: Option<message::StanzaId>,
    pub target: Option<message::Id>,
    pub to: BareJid,
    pub from: BareJid,
    pub timestamp: DateTime<Utc>,
    pub payload: Payload,
    pub is_first_message: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum Payload {
    Correction { body: String },
    DeliveryReceipt,
    ReadReceipt,
    Message { body: String },
    Reaction { emojis: Vec<Emoji> },
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

impl TryFrom<TimestampedMessage<ReceivedMessage<'_>>> for MessageLike {
    type Error = anyhow::Error;

    fn try_from(envelope: TimestampedMessage<ReceivedMessage<'_>>) -> anyhow::Result<Self> {
        match envelope.message {
            ReceivedMessage::Message(msg) => MessageLike::try_from(TimestampedMessage {
                message: msg,
                timestamp: envelope.timestamp,
            }),
            ReceivedMessage::ReceivedCarbon(carbon) => {
                let stanza_id = carbon.message().and_then(|m| m.stanza_id());
                MessageLike::try_from((stanza_id, carbon))
            }
            ReceivedMessage::SentCarbon(carbon) => {
                let stanza_id = carbon.message().and_then(|m| m.stanza_id());
                MessageLike::try_from((stanza_id, carbon))
            }
        }
    }
}

impl TryFrom<TimestampedMessage<&Message<'_>>> for MessageLike {
    type Error = anyhow::Error;

    fn try_from(envelope: TimestampedMessage<&Message<'_>>) -> anyhow::Result<Self> {
        let msg = envelope.message;

        let id = msg
            .id()
            .ok_or(StanzaParseError::missing_attribute("id", msg))?;
        let stanza_id = msg.stanza_id();
        let from = msg
            .from()
            .ok_or(StanzaParseError::missing_attribute("from", msg))?;
        let to = msg
            .to()
            .ok_or(StanzaParseError::missing_attribute("to", msg))?;
        let timestamp = msg
            .delay()
            .and_then(|delay| delay.stamp())
            .unwrap_or(envelope.timestamp);
        let TargetedPayload {
            target: refs,
            payload,
        } = TargetedPayload::try_from(msg)?;

        Ok(MessageLike {
            id,
            stanza_id,
            target: refs,
            to: BareJid::from(to),
            from: BareJid::from(from),
            timestamp,
            payload,
            is_first_message: false,
        })
    }
}

impl TryFrom<&ArchivedMessage<'_>> for MessageLike {
    type Error = anyhow::Error;

    fn try_from(carbon: &ArchivedMessage<'_>) -> Result<Self, Self::Error> {
        MessageLike::try_from((carbon.stanza_id.clone(), &carbon.message))
    }
}

impl TryFrom<(Option<StanzaId>, &ForwardedMessage<'_>)> for MessageLike {
    type Error = anyhow::Error;

    fn try_from(value: (Option<StanzaId>, &ForwardedMessage<'_>)) -> Result<Self, Self::Error> {
        let Some(stanza_id) = value.0 else {
            return Err(anyhow::format_err!("Missing stanza_id in ForwardedMessage"))
        };
        let carbon = value.1;

        let message = carbon
            .message()
            .ok_or(StanzaParseError::missing_child_node("message", carbon))?;
        let id = message
            .id()
            .ok_or(StanzaParseError::missing_attribute("id", &message))?;
        let to = message
            .to()
            .ok_or(StanzaParseError::missing_attribute("to", &message))?;
        let from = message
            .from()
            .ok_or(StanzaParseError::missing_attribute("from", &message))?;
        let timestamp = carbon
            .delay()
            .ok_or(StanzaParseError::missing_child_node("delay", carbon))?
            .stamp()
            .ok_or(StanzaParseError::missing_attribute("stamp", carbon))?;
        let TargetedPayload {
            target: refs,
            payload,
        } = TargetedPayload::try_from(&message)?;

        Ok(MessageLike {
            id,
            stanza_id: Some(stanza_id),
            target: refs,
            to: BareJid::from(to),
            from: BareJid::from(from),
            timestamp,
            payload,
            is_first_message: false,
        })
    }
}

struct TargetedPayload {
    target: Option<message::Id>,
    payload: Payload,
}

impl TryFrom<&Message<'_>> for TargetedPayload {
    type Error = anyhow::Error;

    fn try_from(message: &Message) -> anyhow::Result<Self> {
        if let Some((id, emojis)) = message.message_reactions() {
            return Ok(TargetedPayload {
                target: Some(id),
                payload: Payload::Reaction {
                    emojis: emojis.into_iter().unique().collect(),
                },
            });
        };

        if let Some(fastening) = message.fastening() {
            if fastening.retract() {
                return Ok(TargetedPayload {
                    target: Some(
                        fastening
                            .id()
                            .ok_or(StanzaParseError::missing_attribute("id", &fastening))?,
                    ),
                    payload: Payload::Retraction,
                });
            }
        }

        if let (Some(replace_id), Some(body)) = (message.replace(), message.body()) {
            return Ok(TargetedPayload {
                target: Some(replace_id),
                payload: Payload::Correction { body },
            });
        }

        if let Some(marker) = message.received_marker() {
            if let Some(id) = marker.id() {
                return Ok(TargetedPayload {
                    target: Some(id),
                    payload: Payload::DeliveryReceipt,
                });
            }
        }

        if let Some(marker) = message.displayed_marker() {
            if let Some(id) = marker.id() {
                return Ok(TargetedPayload {
                    target: Some(id),
                    payload: Payload::ReadReceipt,
                });
            }
        }

        if let Some(body) = message.body() {
            return Ok(TargetedPayload {
                target: None,
                payload: Payload::Message { body },
            });
        }

        Err(anyhow::format_err!(
            "Failed to interpret message {:?}",
            message
        ))
    }
}
