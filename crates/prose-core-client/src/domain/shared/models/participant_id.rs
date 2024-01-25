// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{OccupantId, UserEndpointId, UserId};
use jid::Jid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
/// Represents the identifier of a user within - what we define as - room. So it could be either a
/// regular UserId (BareJid) in a DirectMessage room (1:1 conversation) or a OccupantId when in a
/// multi-user room (MUC chat).
pub enum ParticipantId {
    User(UserId),
    Occupant(OccupantId),
}

impl ParticipantId {
    pub fn to_user_id(&self) -> Option<UserId> {
        let ParticipantId::User(id) = &self else {
            return None;
        };
        Some(id.clone())
    }

    pub fn to_occupant_id(&self) -> Option<OccupantId> {
        let ParticipantId::Occupant(id) = &self else {
            return None;
        };
        Some(id.clone())
    }

    pub fn to_opaque_identifier(&self) -> String {
        match self {
            ParticipantId::User(id) => id.to_string(),
            ParticipantId::Occupant(id) => id.to_string(),
        }
    }
}

impl From<UserId> for ParticipantId {
    fn from(value: UserId) -> Self {
        ParticipantId::User(value)
    }
}

impl From<OccupantId> for ParticipantId {
    fn from(value: OccupantId) -> Self {
        ParticipantId::Occupant(value)
    }
}

impl From<UserEndpointId> for ParticipantId {
    fn from(value: UserEndpointId) -> Self {
        match value {
            UserEndpointId::User(id) => id.into(),
            UserEndpointId::UserResource(id) => id.into_user_id().into(),
            UserEndpointId::Occupant(id) => id.into(),
        }
    }
}

impl From<ParticipantId> for Jid {
    fn from(value: ParticipantId) -> Self {
        match value {
            ParticipantId::User(id) => Jid::Bare(id.into_inner()),
            ParticipantId::Occupant(id) => Jid::Full(id.into_inner()),
        }
    }
}
