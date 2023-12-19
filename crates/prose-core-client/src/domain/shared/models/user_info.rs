// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{Availability, UserId};
use crate::domain::rooms::models::{Participant, RoomAffiliation};

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

#[derive(Debug, Clone, PartialEq)]
pub struct ParticipantInfo {
    pub id: Option<UserId>,
    pub name: String,
    pub is_self: bool,
    pub availability: Availability,
    pub affiliation: RoomAffiliation,
}

impl From<&Participant> for ParticipantInfo {
    fn from(value: &Participant) -> Self {
        ParticipantInfo {
            id: value.real_id.clone(),
            name: value.name.as_deref().unwrap_or("<anonymous>").to_string(),
            is_self: value.is_self,
            availability: value.availability.clone(),
            affiliation: value.affiliation.clone(),
        }
    }
}
