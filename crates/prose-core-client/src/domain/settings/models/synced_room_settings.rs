// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};

use crate::domain::messaging::models::ArchivedMessageRef;
use crate::dtos::RoomId;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SyncedRoomSettings {
    pub room_id: RoomId,
    pub encryption_enabled: bool,
    pub last_read_message: Option<ArchivedMessageRef>,
}

impl SyncedRoomSettings {
    pub fn new(room_id: RoomId) -> Self {
        Self {
            room_id,
            encryption_enabled: false,
            last_read_message: Default::default(),
        }
    }
}
