// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::rooms::models::{Participant, RoomAffiliation};

use super::{Availability, ParticipantId, UserId};

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

impl From<(&ParticipantId, &Participant)> for ParticipantInfo {
    fn from(value: (&ParticipantId, &Participant)) -> Self {
        let (id, participant) = value;

        let name = participant.name.clone().unwrap_or_else(|| match id {
            ParticipantId::User(id) => id.formatted_username(),
            ParticipantId::Occupant(id) => participant
                .real_id
                .as_ref()
                .map(|real_id| real_id.formatted_username())
                .unwrap_or_else(|| id.formatted_nickname()),
        });

        ParticipantInfo {
            id: participant.real_id.clone(),
            name,
            is_self: participant.is_self,
            availability: participant.availability,
            affiliation: participant.affiliation,
        }
    }
}
