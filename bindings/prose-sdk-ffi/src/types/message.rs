// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime as ChronoDateTime, Utc};

use prose_core_client::dtos::{Emoji, Message as ProseMessage, MessageId};

use crate::types::JID;

pub type DateTime = ChronoDateTime<Utc>;

#[derive(uniffi::Record)]
pub struct Reaction {
    pub emoji: Emoji,
    pub from: Vec<String>,
}

#[derive(uniffi::Record)]
pub struct Message {
    pub id: MessageId,
    pub from: Option<JID>,
    pub body: String,
    pub timestamp: DateTime,
    pub is_read: bool,
    pub is_edited: bool,
    pub is_delivered: bool,
    // pub reactions: Vec<Reaction>,
}

impl From<ProseMessage> for Message {
    fn from(value: ProseMessage) -> Self {
        Message {
            id: value.id,
            from: value.from.id.to_user_id().map(|id| id.into_inner().into()),
            body: todo!(),
            timestamp: value.timestamp,
            is_read: value.is_read,
            is_edited: value.is_edited,
            is_delivered: value.is_delivered,
            // reactions: value.reactions.into_iter().map(Into::into).collect(),
        }
    }
}

// impl From<ProseReaction> for Reaction {
//     fn from(value: ProseReaction) -> Self {
//         Reaction {
//             emoji: value.emoji,
//             from: value
//                 .from
//                 .into_iter()
//                 .map(|id| id.to_opaque_identifier())
//                 .collect(),
//         }
//     }
// }
