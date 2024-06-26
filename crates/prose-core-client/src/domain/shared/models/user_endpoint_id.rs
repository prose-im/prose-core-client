// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::Jid;

use super::{OccupantId, ParticipantId, RoomId, UserId, UserOrResourceId, UserResourceId};

// Represents any id a user can be identified by.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserEndpointId {
    User(UserId),
    UserResource(UserResourceId),
    Occupant(OccupantId),
}

impl UserEndpointId {
    pub fn to_room_id(&self) -> RoomId {
        match self {
            UserEndpointId::User(id) => RoomId::from(id.clone()),
            UserEndpointId::UserResource(id) => RoomId::from(id.to_user_id()),
            UserEndpointId::Occupant(id) => RoomId::from(id.muc_id()),
        }
    }

    pub fn to_participant_id(&self) -> ParticipantId {
        match self {
            UserEndpointId::User(id) => ParticipantId::User(id.clone()),
            UserEndpointId::UserResource(id) => ParticipantId::User(id.to_user_id()),
            UserEndpointId::Occupant(id) => ParticipantId::Occupant(id.clone()),
        }
    }

    pub fn to_user_or_resource_id(&self) -> Option<UserOrResourceId> {
        match self {
            UserEndpointId::User(id) => Some(UserOrResourceId::User(id.clone())),
            UserEndpointId::UserResource(id) => Some(UserOrResourceId::UserResource(id.clone())),
            UserEndpointId::Occupant(_) => None,
        }
    }

    pub fn to_user_id(&self) -> Option<UserId> {
        match self {
            UserEndpointId::User(id) => Some(id.clone()),
            UserEndpointId::UserResource(id) => Some(id.to_user_id()),
            UserEndpointId::Occupant(_) => None,
        }
    }

    pub fn is_occupant_id(&self) -> bool {
        match self {
            UserEndpointId::Occupant(_) => true,
            UserEndpointId::User(_) | UserEndpointId::UserResource(_) => false,
        }
    }

    pub fn into_jid(self) -> Jid {
        match self {
            UserEndpointId::User(id) => Jid::from(id.into_inner()),
            UserEndpointId::UserResource(id) => Jid::from(id.into_inner()),
            UserEndpointId::Occupant(id) => Jid::from(id.into_inner()),
        }
    }
}

impl From<UserId> for UserEndpointId {
    fn from(value: UserId) -> Self {
        Self::User(value)
    }
}

impl From<UserResourceId> for UserEndpointId {
    fn from(value: UserResourceId) -> Self {
        Self::UserResource(value)
    }
}

impl From<OccupantId> for UserEndpointId {
    fn from(value: OccupantId) -> Self {
        Self::Occupant(value)
    }
}
