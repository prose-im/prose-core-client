use crate::types::JID;
use chrono::{DateTime as ChronoDateTime, Utc};
use prose_core_client::types::{
    Emoji, Message as ProseMessage, MessageId, Reaction as ProseReaction, StanzaId,
};

pub type DateTime = ChronoDateTime<Utc>;

#[derive(uniffi::Record)]
pub struct Reaction {
    pub emoji: Emoji,
    pub from: Vec<JID>,
}

#[derive(uniffi::Record)]
pub struct Message {
    pub id: MessageId,
    pub stanza_id: Option<StanzaId>,
    pub from: JID,
    pub body: String,
    pub timestamp: DateTime,
    pub is_read: bool,
    pub is_edited: bool,
    pub is_delivered: bool,
    pub reactions: Vec<Reaction>,
}

impl From<ProseMessage> for Message {
    fn from(value: ProseMessage) -> Self {
        Message {
            id: value.id,
            stanza_id: value.stanza_id,
            from: value.from.into(),
            body: value.body,
            timestamp: value.timestamp,
            is_read: value.is_read,
            is_edited: value.is_edited,
            is_delivered: value.is_delivered,
            reactions: value.reactions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ProseReaction> for Reaction {
    fn from(value: ProseReaction) -> Self {
        Reaction {
            emoji: value.emoji,
            from: value.from.into_iter().map(Into::into).collect(),
        }
    }
}
