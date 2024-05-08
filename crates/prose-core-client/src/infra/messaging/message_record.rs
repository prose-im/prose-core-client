// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use jid::BareJid;
use serde::{Deserialize, Serialize};

use prose_store::prelude::Entity;
use prose_store::{IndexSpec, KeyType, RawKey};

use crate::domain::messaging::models::{
    MessageLike, MessageLikeId, MessageLikePayload, MessageTargetId,
};
use crate::dtos::{MessageId, ParticipantId, StanzaId};

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageRecord {
    pub id: MessageLikeId,
    pub stanza_id: Option<StanzaId>,
    pub stanza_id_target: Option<StanzaId>,
    pub message_id_target: Option<MessageId>,
    pub to: Option<BareJid>,
    pub from: ParticipantId,
    pub timestamp: DateTime<Utc>,
    pub payload: MessageLikePayload,
}

impl MessageRecord {
    pub fn stanza_id_target_idx() -> &'static str {
        "stanza_id_target"
    }

    pub fn message_id_target_idx() -> &'static str {
        "message_id_target"
    }

    pub fn stanza_id_idx() -> &'static str {
        "stanza_id"
    }
}

impl Entity for MessageRecord {
    type ID = MessageLikeId;

    fn id(&self) -> &Self::ID {
        &self.id
    }

    fn collection() -> &'static str {
        "messages"
    }

    fn indexes() -> Vec<IndexSpec> {
        vec![
            IndexSpec::builder()
                .add_column(Self::stanza_id_idx())
                .build(),
            IndexSpec::builder()
                .add_column(Self::stanza_id_target_idx())
                .build(),
            IndexSpec::builder()
                .add_column(Self::message_id_target_idx())
                .build(),
        ]
    }
}

impl KeyType for MessageLikeId {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_string())
    }
}

impl KeyType for MessageId {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_string())
    }
}

impl KeyType for StanzaId {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_string())
    }
}

impl From<MessageLike> for MessageRecord {
    fn from(value: MessageLike) -> Self {
        let (stanza_id_target, message_id_target) = match value.target {
            Some(MessageTargetId::MessageId(id)) => (None, Some(id)),
            Some(MessageTargetId::StanzaId(id)) => (Some(id), None),
            None => (None, None),
        };

        Self {
            id: value.id,
            stanza_id: value.stanza_id,
            stanza_id_target,
            message_id_target,
            to: value.to,
            from: value.from,
            timestamp: value.timestamp,
            payload: value.payload,
        }
    }
}

impl From<MessageRecord> for MessageLike {
    fn from(value: MessageRecord) -> Self {
        let target = match (value.stanza_id_target, value.message_id_target) {
            (Some(id), _) => Some(MessageTargetId::StanzaId(id)),
            (_, Some(id)) => Some(MessageTargetId::MessageId(id)),
            (None, None) => None,
        };

        Self {
            id: value.id,
            stanza_id: value.stanza_id,
            target,
            to: value.to,
            from: value.from,
            timestamp: value.timestamp,
            payload: value.payload,
        }
    }
}
