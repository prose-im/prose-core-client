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
use crate::dtos::{MessageId, ParticipantId, RoomId, StanzaId, UserId};

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageRecord {
    pub id: String,
    pub account: UserId,
    pub room_id: RoomId,
    pub message_id: MessageLikeId,
    pub message_id_target: Option<MessageId>,
    pub stanza_id: Option<StanzaId>,
    pub stanza_id_target: Option<StanzaId>,
    pub to: Option<BareJid>,
    pub from: ParticipantId,
    pub timestamp: DateTime<Utc>,
    pub payload: MessageLikePayload,
}

mod columns {
    pub const ACCOUNT: &str = "account";
    pub const ROOM_ID: &str = "room_id";
    pub const STANZA_ID: &str = "stanza_id";
    pub const STANZA_ID_TARGET: &str = "stanza_id_target";
    pub const MESSAGE_ID: &str = "message_id";
    pub const MESSAGE_ID_TARGET: &str = "message_id_target";
}

impl MessageRecord {
    pub fn account_idx() -> [&'static str; 1] {
        [columns::ACCOUNT]
    }

    pub fn stanza_id_idx() -> [&'static str; 3] {
        [columns::ACCOUNT, columns::ROOM_ID, columns::STANZA_ID]
    }

    pub fn stanza_id_target_idx() -> [&'static str; 3] {
        [
            columns::ACCOUNT,
            columns::ROOM_ID,
            columns::STANZA_ID_TARGET,
        ]
    }

    pub fn message_id_idx() -> [&'static str; 3] {
        [columns::ACCOUNT, columns::ROOM_ID, columns::MESSAGE_ID]
    }

    pub fn message_id_target_idx() -> [&'static str; 3] {
        [
            columns::ACCOUNT,
            columns::ROOM_ID,
            columns::MESSAGE_ID_TARGET,
        ]
    }
}

impl Entity for MessageRecord {
    type ID = String;

    fn id(&self) -> &Self::ID {
        &self.id
    }

    fn collection() -> &'static str {
        "messages"
    }

    fn indexes() -> Vec<IndexSpec> {
        vec![
            IndexSpec::builder().add_column(columns::ACCOUNT).build(),
            IndexSpec::builder()
                .add_column(columns::ACCOUNT)
                .add_column(columns::ROOM_ID)
                .add_column(columns::STANZA_ID)
                .unique()
                .build(),
            IndexSpec::builder()
                .add_column(columns::ACCOUNT)
                .add_column(columns::ROOM_ID)
                .add_column(columns::STANZA_ID_TARGET)
                .build(),
            IndexSpec::builder()
                .add_column(columns::ACCOUNT)
                .add_column(columns::ROOM_ID)
                .add_column(columns::MESSAGE_ID)
                .unique()
                .build(),
            IndexSpec::builder()
                .add_column(columns::ACCOUNT)
                .add_column(columns::ROOM_ID)
                .add_column(columns::MESSAGE_ID_TARGET)
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

impl MessageRecord {
    pub fn from_message(account: UserId, room_id: RoomId, value: MessageLike) -> Self {
        let (stanza_id_target, message_id_target) = match value.target {
            Some(MessageTargetId::MessageId(id)) => (None, Some(id)),
            Some(MessageTargetId::StanzaId(id)) => (Some(id), None),
            None => (None, None),
        };

        let id = format!("{account}-{}-{}", room_id.to_raw_key_string(), value.id);

        Self {
            account,
            room_id,
            id,
            message_id: value.id,
            message_id_target,
            stanza_id: value.stanza_id,
            stanza_id_target,
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
            id: value.message_id,
            stanza_id: value.stanza_id,
            target,
            to: value.to,
            from: value.from,
            timestamp: value.timestamp,
            payload: value.payload,
        }
    }
}
