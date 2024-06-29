// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::rooms::models::{Participant, RoomAffiliation};
use crate::domain::user_info::models::{Avatar, JabberClient};

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
    pub avatar: Option<Avatar>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParticipantInfo {
    pub id: Option<UserId>,
    pub name: String,
    pub is_self: bool,
    pub availability: Availability,
    pub affiliation: RoomAffiliation,
    pub avatar: Option<Avatar>,
    pub client: Option<JabberClient>,
}

impl From<(&ParticipantId, &Participant)> for ParticipantInfo {
    fn from(value: (&ParticipantId, &Participant)) -> Self {
        let (id, participant) = value;

        ParticipantInfo {
            id: participant.real_id.clone(),
            name: participant.name().or_participant_id(id).into_string(),
            is_self: participant.is_self,
            availability: participant.availability,
            affiliation: participant.affiliation,
            avatar: participant.avatar.clone(),
            client: participant.client.clone(),
        }
    }
}
