// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

use jid::BareJid;
use minidom::IntoAttributeValue;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Represents a unique XMPP user identifier without resource specification.
pub struct UserId(BareJid);

impl UserId {
    pub fn into_inner(self) -> BareJid {
        self.0
    }
}

impl From<BareJid> for UserId {
    fn from(value: BareJid) -> Self {
        UserId(value)
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl IntoAttributeValue for UserId {
    fn into_attribute_value(self) -> Option<String> {
        self.0.into_attribute_value()
    }
}
