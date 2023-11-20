// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;

use crate::domain::shared::models::{RoomJid, RoomType};

/// Contains information about a room after creating or joining it.
#[derive(Debug, PartialEq, Clone)]
pub struct RoomSessionInfo {
    pub room_jid: RoomJid,
    pub room_name: Option<String>,
    pub room_description: Option<String>,
    pub room_type: RoomType,
    pub user_nickname: String,
    pub members: Vec<BareJid>,
    pub room_has_been_created: bool,
}
