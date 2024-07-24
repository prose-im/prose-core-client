// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use jid::BareJid;
use serde::{Deserialize, Serialize};

use prose_store::prelude::*;

use crate::domain::messaging::models::{
    MessageLike, MessageLikeId, MessageLikePayload, MessageTargetId,
};
use crate::domain::shared::models::AccountId;
use crate::dtos::{MessageRemoteId, MessageServerId, ParticipantId, RoomId};

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageRecord {
    pub id: String,
    pub account: AccountId,
    pub room_id: RoomId,
    pub message_id: MessageLikeId,
    pub message_id_target: Option<MessageRemoteId>,
    pub stanza_id: Option<MessageServerId>,
    pub stanza_id_target: Option<MessageServerId>,
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
    pub const TIMESTAMP: &str = "timestamp";
}

define_entity!(MessageRecord, "messages",
    account_idx => { columns: [columns::ACCOUNT], unique: false },
    room_idx => { columns: [columns::ACCOUNT, columns::ROOM_ID], unique: false },
    // Can't be unique, because stanzaId might be nil…
    stanza_id_idx => { columns: [columns::ACCOUNT, columns::ROOM_ID, columns::STANZA_ID], unique: false },
    stanza_id_target_idx => { columns: [columns::ACCOUNT, columns::ROOM_ID, columns::STANZA_ID_TARGET], unique: false },
    message_id_idx => { columns: [columns::ACCOUNT, columns::ROOM_ID, columns::MESSAGE_ID], unique: true },
    message_id_target_idx => { columns: [columns::ACCOUNT, columns::ROOM_ID, columns::MESSAGE_ID_TARGET], unique: false },
    timestamp_idx => { columns: [columns::ACCOUNT, columns::ROOM_ID, columns::TIMESTAMP], unique: false }
);

impl KeyType for MessageLikeId {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_string())
    }
}

impl KeyType for MessageRemoteId {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_string())
    }
}

impl KeyType for MessageServerId {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_string())
    }
}

impl MessageRecord {
    pub fn from_message(account: AccountId, room_id: RoomId, value: MessageLike) -> Self {
        let (stanza_id_target, message_id_target) = match value.target {
            Some(MessageTargetId::RemoteId(id)) => (None, Some(id)),
            Some(MessageTargetId::ServerId(id)) => (Some(id), None),
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
            (Some(id), _) => Some(MessageTargetId::ServerId(id)),
            (_, Some(id)) => Some(MessageTargetId::RemoteId(id)),
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
