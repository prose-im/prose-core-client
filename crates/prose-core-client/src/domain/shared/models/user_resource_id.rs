// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Display, Formatter};

use jid::{FullJid, Jid};
use minidom::IntoAttributeValue;
use serde::{Deserialize, Serialize};

use super::UserId;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Represents a unique XMPP user identifier including the specific resource part.
pub struct UserResourceId(FullJid);

impl UserResourceId {
    pub fn into_inner(self) -> FullJid {
        self.0
    }

    pub fn to_user_id(&self) -> UserId {
        UserId::from(self.0.to_bare())
    }

    pub fn into_user_id(self) -> UserId {
        UserId::from(self.0.into_bare())
    }

    pub fn resource(&self) -> &str {
        &self.0.resource_str()
    }

    pub fn username(&self) -> &str {
        self.0.node_str().expect("Missing node in UserId")
    }
}

impl From<FullJid> for UserResourceId {
    fn from(value: FullJid) -> Self {
        assert!(value.node_str().is_some(), "Missing node in UserResourceId");
        UserResourceId(value)
    }
}

impl Debug for UserResourceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "UserResourceId({})", self.0)
    }
}

impl Display for UserResourceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl IntoAttributeValue for UserResourceId {
    fn into_attribute_value(self) -> Option<String> {
        self.0.into_attribute_value()
    }
}

impl AsRef<FullJid> for UserResourceId {
    fn as_ref(&self) -> &FullJid {
        &self.0
    }
}

impl From<UserResourceId> for Jid {
    fn from(value: UserResourceId) -> Self {
        Jid::Full(value.0)
    }
}
