// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Display, Formatter};

use crate::dtos::UserId;

pub struct PresenceSubRequest {
    pub id: PresenceSubRequestId,
    pub name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PresenceSubRequestId(UserId);

impl From<UserId> for PresenceSubRequestId {
    fn from(value: UserId) -> Self {
        Self(value)
    }
}

impl PresenceSubRequestId {
    pub(crate) fn to_user_id(&self) -> UserId {
        self.0.clone()
    }
}

impl Display for PresenceSubRequestId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
