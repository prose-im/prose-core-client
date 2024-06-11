// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::rooms::models::RoomAffiliation;
use crate::domain::shared::models::{
    AnonOccupantId, Availability, MamVersion, MucId, OccupantId, RoomType, UserId,
};

/// Contains information about a room after creating or joining it.
#[derive(Debug, PartialEq, Clone)]
pub struct RoomSessionInfo {
    pub room_id: MucId,
    pub config: RoomConfig,
    pub topic: Option<String>,
    pub user_nickname: String,
    pub members: Vec<RoomSessionMember>,
    pub participants: Vec<RoomSessionParticipant>,
    pub room_has_been_created: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RoomConfig {
    pub room_name: Option<String>,
    pub room_description: Option<String>,
    pub room_type: RoomType,
    pub mam_version: Option<MamVersion>,
    pub supports_self_ping_optimization: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RoomSessionMember {
    pub id: UserId,
    pub affiliation: RoomAffiliation,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RoomSessionParticipant {
    pub id: OccupantId,
    pub is_self: bool,
    pub anon_id: Option<AnonOccupantId>,
    pub real_id: Option<UserId>,
    pub affiliation: RoomAffiliation,
    pub availability: Availability,
}
