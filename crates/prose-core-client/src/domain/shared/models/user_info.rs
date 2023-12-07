// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{Availability, UserId};
use crate::domain::rooms::models::RoomAffiliation;

#[derive(Debug, Clone, PartialEq)]
pub struct UserBasicInfo {
    pub id: UserId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserPresenceInfo {
    pub id: UserId,
    pub name: String,
    pub availability: Availability,
}

pub struct ParticipantInfo {
    pub id: Option<UserId>,
    pub name: String,
    pub availability: Availability,
    pub affiliation: RoomAffiliation,
}
