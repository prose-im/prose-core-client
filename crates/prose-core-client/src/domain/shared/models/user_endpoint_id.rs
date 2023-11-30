// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{OccupantId, UserId, UserResourceId};

// Represents any id a user can identified by.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserEndpointId {
    User(UserId),
    UserResource(UserResourceId),
    Occupant(OccupantId),
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
