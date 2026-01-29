// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{Availability, ParticipantId, UserId};
use crate::domain::rooms::models::{Participant, RoomAffiliation};
use crate::domain::shared::models::{Avatar, AvatarBundle};
use crate::domain::user_info::models::JabberClient;
use crate::dtos::UserStatus;

#[derive(Debug, Clone, PartialEq)]
pub struct UserBasicInfo {
    pub id: UserId,
    pub name: String,
    pub avatar: Option<Avatar>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserPresenceInfo {
    pub id: UserId,
    pub name: String,
    pub full_name: Option<String>,
    pub availability: Availability,
    pub avatar: Option<Avatar>,
    pub status: Option<UserStatus>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticipantBasicInfo {
    pub id: ParticipantId,
    pub name: String,
    pub avatar: Option<Avatar>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticipantInfo {
    pub id: ParticipantId,
    pub user_id: Option<UserId>,
    pub name: String,
    pub is_self: bool,
    pub availability: Availability,
    pub affiliation: RoomAffiliation,
    pub avatar: Option<Avatar>,
    pub client: Option<JabberClient>,
    pub status: Option<String>,
}

impl From<(&ParticipantId, &Participant)> for ParticipantInfo {
    fn from(value: (&ParticipantId, &Participant)) -> Self {
        let (id, participant) = value;

        ParticipantInfo {
            id: id.clone(),
            user_id: participant.real_id.clone(),
            name: participant.name().unwrap_or_participant_id(id),
            is_self: participant.is_self,
            availability: participant.availability,
            affiliation: participant.affiliation,
            avatar: participant.avatar.clone(),
            client: participant.client.clone(),
            status: participant.status.clone(),
        }
    }
}

impl ParticipantBasicInfo {
    pub fn avatar_bundle(&self) -> AvatarBundle {
        AvatarBundle::with_generated_initials_and_color(&self.id, &self.name, self.avatar.as_ref())
    }
}

impl ParticipantInfo {
    pub fn avatar_bundle(&self) -> AvatarBundle {
        AvatarBundle::with_generated_initials_and_color(&self.id, &self.name, self.avatar.as_ref())
    }
}

impl UserPresenceInfo {
    pub fn avatar_bundle(&self) -> AvatarBundle {
        AvatarBundle::with_generated_initials_and_color(&self.id, &self.name, self.avatar.as_ref())
    }
}
