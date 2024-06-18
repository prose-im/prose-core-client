// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use jid::{BareJid, Jid};
use minidom::IntoAttributeValue;
use serde::{Deserialize, Serialize};

use prose_store::{KeyType, RawKey};

use crate::infra::xmpp::util::{JidExt, JidParseError};
use crate::util::StringExt;

use super::UserResourceId;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Represents a unique XMPP user identifier without resource specification.
pub struct UserId(BareJid);

impl UserId {
    pub fn into_inner(self) -> BareJid {
        self.0
    }

    pub fn with_resource(&self, res: impl AsRef<str>) -> Result<UserResourceId, jid::Error> {
        Ok(UserResourceId::from(
            self.0.with_resource_str(res.as_ref())?,
        ))
    }

    pub fn username(&self) -> &str {
        self.0.node().expect("Missing node in UserId")
    }

    pub fn formatted_username(&self) -> String {
        let Some(node) = self.0.node() else {
            return self.to_string().to_uppercase_first_letter();
        };
        node.capitalized_display_name()
    }

    pub fn is_same_domain(&self, other: &UserId) -> bool {
        self.0.domain() == other.0.domain()
    }
}

impl From<BareJid> for UserId {
    fn from(value: BareJid) -> Self {
        assert!(value.node().is_some(), "Missing node in UserId");
        UserId(value)
    }
}

impl Debug for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "UserId({})", self.0)
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

impl UserId {
    pub fn from_iri(iri: &str) -> Result<Self, JidParseError> {
        Ok(Self(Jid::from_iri(iri)?.into_bare()))
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

impl PartialOrd for UserId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UserId {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = self.username().cmp(other.username());
        if ord != Ordering::Equal {
            return ord;
        }
        self.0.domain().cmp(other.0.domain())
    }
}
