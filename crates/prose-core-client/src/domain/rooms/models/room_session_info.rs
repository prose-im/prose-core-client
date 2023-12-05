// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::rooms::models::RoomAffiliation;
use crate::domain::shared::models::{RoomId, RoomType, UserId};

/// Contains information about a room after creating or joining it.
#[derive(Debug, PartialEq, Clone)]
pub struct RoomSessionInfo {
    pub room_jid: RoomId,
    pub room_name: Option<String>,
    pub room_description: Option<String>,
    pub room_type: RoomType,
    pub user_nickname: String,
    pub members: Vec<RoomSessionMember>,
    pub room_has_been_created: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RoomSessionMember {
    pub id: UserId,
    pub affiliation: RoomAffiliation,
    // Only available if member is currently an occupant.
    pub nick: Option<String>,
}
