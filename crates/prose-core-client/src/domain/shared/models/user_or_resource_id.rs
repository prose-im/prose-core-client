// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::{UserId, UserResourceId};

pub enum UserOrResourceId {
    User(UserId),
    UserResource(UserResourceId),
}

impl UserOrResourceId {
    pub fn to_user_id(&self) -> UserId {
        match self {
            UserOrResourceId::User(id) => id.clone(),
            UserOrResourceId::UserResource(id) => id.to_user_id(),
        }
    }

    pub fn resource_str(&self) -> Option<&str> {
        match self {
            UserOrResourceId::User(_) => None,
            UserOrResourceId::UserResource(id) => Some(id.resource_str()),
        }
    }
}

impl From<UserId> for UserOrResourceId {
    fn from(value: UserId) -> Self {
        Self::User(value)
    }
}

impl From<UserResourceId> for UserOrResourceId {
    fn from(value: UserResourceId) -> Self {
        Self::UserResource(value)
    }
}
