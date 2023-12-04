// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use jid::BareJid;
use minidom::IntoAttributeValue;
use serde::{Deserialize, Serialize};

use prose_store::{KeyType, RawKey};

use crate::util::StringExt;

use super::UserResourceId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Represents a unique XMPP user identifier without resource specification.
pub struct UserId(BareJid);

impl UserId {
    pub fn into_inner(self) -> BareJid {
        self.0
    }

    pub fn with_resource(&self, res: impl AsRef<str>) -> Result<UserResourceId, jid::Error> {
        Ok(UserResourceId::from(self.0.with_resource_str(res.as_ref())?))
    }

    pub fn username(&self) -> Option<&str> {
        self.0.node_str()
    }

    pub fn formatted_username(&self) -> String {
        let Some(node) = self.0.node_str() else {
            return self.to_string().to_uppercase_first_letter();
        };
        node.capitalized_display_name()
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

impl FromStr for UserId {
    type Err = jid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UserId(s.parse::<BareJid>()?))
    }
}

impl AsRef<BareJid> for UserId {
    fn as_ref(&self) -> &BareJid {
        &self.0
    }
}

impl From<UserId> for BareJid {
    fn from(value: UserId) -> Self {
        value.0
    }
}

impl IntoAttributeValue for UserId {
    fn into_attribute_value(self) -> Option<String> {
        self.0.into_attribute_value()
    }
}

impl KeyType for UserId {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.0.to_string())
    }
}

impl KeyType for &UserId {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.0.to_string())
    }
}
