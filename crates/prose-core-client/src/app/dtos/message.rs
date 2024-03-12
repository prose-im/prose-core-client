// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};

use crate::domain::shared::models::ParticipantId;
use crate::dtos::{Attachment, Mention, MessageId, Reaction, StanzaId};

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub id: Option<MessageId>,
    pub stanza_id: Option<StanzaId>,
    pub from: MessageSender,
    pub body: String,
    pub timestamp: DateTime<Utc>,
    pub is_read: bool,
    pub is_edited: bool,
    pub is_delivered: bool,
    pub is_transient: bool,
    pub is_encrypted: bool,
    pub reactions: Vec<Reaction>,
    pub attachments: Vec<Attachment>,
    pub mentions: Vec<Mention>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageSender {
    pub id: ParticipantId,
    pub name: String,
}
