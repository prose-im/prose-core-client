// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use jid::BareJid;
use serde::{Deserialize, Serialize};

use prose_store::prelude::*;

use crate::domain::messaging::models::{
    MessageId, MessageLike, MessageLikePayload, MessageTargetId,
};
use crate::domain::shared::models::AccountId;
use crate::dtos::{MessageRemoteId, MessageServerId, ParticipantId, RoomId};

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageRecord {
    pub id: String,
    pub account: AccountId,
    pub room_id: RoomId,
    pub message_id: MessageId,
    pub remote_id: Option<MessageRemoteId>,
    pub remote_id_target: Option<MessageRemoteId>,
    pub server_id: Option<MessageServerId>,
    pub server_id_target: Option<MessageServerId>,
    pub to: Option<BareJid>,
    pub from: ParticipantId,
    pub timestamp: DateTime<Utc>,
    pub payload: MessageLikePayload,
}

mod columns {
    pub const ACCOUNT: &str = "account";
    pub const ROOM_ID: &str = "room_id";
    pub const MESSAGE_ID: &str = "message_id";
    pub const SERVER_ID: &str = "server_id";
    pub const SERVER_ID_TARGET: &str = "server_id_target";
    pub const REMOTE_ID: &str = "remote_id";
    pub const REMOTE_ID_TARGET: &str = "remote_id_target";
    pub const TIMESTAMP: &str = "timestamp";
}

define_entity!(MessageRecord, "messages",
    account_idx => { columns: [columns::ACCOUNT], unique: false },
    room_idx => { columns: [columns::ACCOUNT, columns::ROOM_ID], unique: false },
    message_id_idx => { columns: [columns::ACCOUNT, columns::ROOM_ID, columns::MESSAGE_ID], unique: true },
    // Can't be unique, because stanzaId might be nil…
    server_id_idx => { columns: [columns::ACCOUNT, columns::ROOM_ID, columns::SERVER_ID], unique: false },
    server_id_target_idx => { columns: [columns::ACCOUNT, columns::ROOM_ID, columns::SERVER_ID_TARGET], unique: false },
    // Can't be unique, because remote ids are not guaranteed to be unique…
    remote_id_idx => { columns: [columns::ACCOUNT, columns::ROOM_ID, columns::REMOTE_ID], unique: false },
    remote_id_target_idx => { columns: [columns::ACCOUNT, columns::ROOM_ID, columns::REMOTE_ID_TARGET], unique: false },
    timestamp_idx => { columns: [columns::ACCOUNT, columns::ROOM_ID, columns::TIMESTAMP], unique: false }
);

impl KeyType for MessageId {
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
        let (stanza_id_target, message_id_target) = match value.payload.target_id() {
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
            remote_id: value.remote_id,
            remote_id_target: message_id_target.cloned(),
            server_id: value.server_id,
            server_id_target: stanza_id_target.cloned(),
            to: value.to,
            from: value.from,
            timestamp: value.timestamp,
            payload: value.payload,
        }
    }
}

impl From<MessageRecord> for MessageLike {
    fn from(value: MessageRecord) -> Self {
        Self {
            id: value.message_id,
            remote_id: value.remote_id,
            server_id: value.server_id,
            to: value.to,
            from: value.from,
            timestamp: value.timestamp,
            payload: value.payload,
        }
    }
}
