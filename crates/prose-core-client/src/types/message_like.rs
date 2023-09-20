// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use jid::BareJid;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xmpp_parsers::Element;

use prose_xmpp::mods::chat::Carbon;
use prose_xmpp::stanza::message;
use prose_xmpp::stanza::message::{mam, stanza_id, Forwarded, Message};

use crate::types::error::StanzaParseError;

/// A type that describes permanent messages, i.e. messages that need to be replayed to restore
/// the complete history of a conversation. Note that ephemeral messages like chat states are
/// handled differently.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MessageLike {
    pub id: MessageLikeId,
    pub stanza_id: Option<stanza_id::Id>,
    pub target: Option<message::Id>,
    pub to: Option<BareJid>,
    pub from: BareJid,
    pub timestamp: DateTime<FixedOffset>,
    pub payload: Payload,
    pub is_first_message: bool,
}

/// A ID that can act as a placeholder in the rare cases when a message doesn't have an ID. Since
/// our DataCache backends require some ID for each message we simply generate one.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MessageLikeId(message::Id);

impl MessageLikeId {
    pub fn new(id: Option<message::Id>) -> Self {
        if let Some(id) = id {
            return MessageLikeId(id);
        }
        return MessageLikeId(format!("!!{}", Uuid::new_v4().to_string()).into());
    }

    /// Returns either the original message ID or the generated one.
    pub fn id(&self) -> &message::Id {
        &self.0
    }

    /// Returns the original message ID or None if we contain a generated ID.
    pub fn into_original_id(self) -> Option<message::Id> {
        if self.0.as_ref().starts_with("!!") {
            return None;
        }
        return Some(self.0);
    }
}

impl std::str::FromStr for MessageLikeId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(MessageLikeId(s.to_string().into()))
    }
}

impl<T> From<T> for MessageLikeId
where
    T: Into<String>,
{
    fn from(s: T) -> MessageLikeId {
        MessageLikeId(message::Id::from(s.into()))
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum Payload {
    Correction { body: String },
    DeliveryReceipt,
    ReadReceipt,
    Message { body: String },
    Reaction { emojis: Vec<message::Emoji> },
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
    pub timestamp: DateTime<FixedOffset>,
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
            .and_then(|s| s.stanza_id.as_ref())
            .map(|sid| sid.id.clone());
        MessageLike::try_from((stanza_id, &carbon))
    }
}

impl TryFrom<TimestampedMessage<Message>> for MessageLike {
    type Error = anyhow::Error;

    fn try_from(envelope: TimestampedMessage<Message>) -> Result<Self> {
        let msg = envelope.message;

        let id = MessageLikeId::new(msg.id.clone());
        let stanza_id = &msg.stanza_id;
        let from = msg
            .from
            .as_ref()
            .ok_or(StanzaParseError::missing_attribute("from"))?;
        let to = msg.to.as_ref();
        let timestamp = msg
            .delay
            .as_ref()
            .map(|delay| delay.stamp.0)
            .unwrap_or(envelope.timestamp);
        let TargetedPayload {
            target: refs,
            payload,
        } = TargetedPayload::try_from(&msg)?;

        Ok(MessageLike {
            id,
            stanza_id: stanza_id.as_ref().map(|s| s.id.clone()),
            target: refs,
            to: to.map(|jid| jid.to_bare()),
            from: from.to_bare(),
            timestamp: timestamp.clone(),
            payload,
            is_first_message: false,
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

        let TargetedPayload {
            target: refs,
            payload,
        } = TargetedPayload::try_from(&message)?;

        let id = MessageLikeId::new(message.id);
        let to = message.to;
        let from = message
            .from
            .ok_or(StanzaParseError::missing_attribute("from"))?;
        let timestamp = &carbon
            .delay
            .as_ref()
            .ok_or(StanzaParseError::missing_child_node("delay"))?
            .stamp;

        Ok(MessageLike {
            id,
            stanza_id: Some(stanza_id),
            target: refs,
            to: to.map(|jid| jid.to_bare()),
            from: from.to_bare(),
            timestamp: timestamp.0,
            payload,
            is_first_message: false,
        })
    }
}

struct TargetedPayload {
    target: Option<message::Id>,
    payload: Payload,
}

impl TryFrom<&Message> for TargetedPayload {
    type Error = anyhow::Error;

    fn try_from(message: &Message) -> Result<Self> {
        if let Some(reactions) = &message.reactions {
            return Ok(TargetedPayload {
                target: Some(reactions.id.clone()),
                payload: Payload::Reaction {
                    emojis: reactions.reactions.clone(),
                },
            });
        };

        if let Some(fastening) = &message.fastening {
            if fastening.retract() {
                return Ok(TargetedPayload {
                    target: Some(fastening.id.clone()),
                    payload: Payload::Retraction,
                });
            }
        }

        if let (Some(replace_id), Some(body)) = (&message.replace, &message.body) {
            return Ok(TargetedPayload {
                target: Some(replace_id.clone()),
                payload: Payload::Correction { body: body.clone() },
            });
        }

        if let Some(marker) = &message.received_marker {
            return Ok(TargetedPayload {
                target: Some(marker.id.clone()),
                payload: Payload::DeliveryReceipt,
            });
        }

        if let Some(marker) = &message.displayed_marker {
            return Ok(TargetedPayload {
                target: Some(marker.id.clone()),
                payload: Payload::ReadReceipt,
            });
        }

        if let Some(body) = &message.body {
            return Ok(TargetedPayload {
                target: None,
                payload: Payload::Message { body: body.clone() },
            });
        }

        let fallback = TargetedPayload {
            target: None,
            payload: Payload::Message {
                body: format!(
                    "Failed to interpret message {}",
                    String::from(&Element::from(message.clone()))
                ),
            },
        };
        Ok(fallback)

        // Err(anyhow::format_err!(
        //     "Failed to interpret message {:?}",
        //     message
        // ))
    }
}
