// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

use jid::FullJid;
use minidom::IntoAttributeValue;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Represents a unique XMPP user identifier including the specific resource part.
pub struct UserResourceId(FullJid);

impl UserResourceId {
    pub fn into_inner(self) -> FullJid {
        self.0
    }
}

impl From<FullJid> for UserResourceId {
    fn from(value: FullJid) -> Self {
        UserResourceId(value)
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
