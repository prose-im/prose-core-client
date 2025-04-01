// prose-core-client/prose-core-client
//
// Copyright: 2025, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::util::StringExt;
use jid::{BareJid, Jid};
use minidom::IntoAttributeValue;
use prose_store::{KeyType, RawKey};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Represents a unique XMPP user identifier without resource specification and without a node.
pub struct ServerId(BareJid);

impl ServerId {
    pub fn into_inner(self) -> BareJid {
        self.0
    }

    pub fn formatted_name(&self) -> String {
        self.0.domain().as_str().capitalized_display_name()
    }
}

impl From<BareJid> for ServerId {
    fn from(value: BareJid) -> Self {
        assert!(
            value.node().is_none(),
            "ServerId is expected to not have a node"
        );
        ServerId(value)
    }
}

impl Debug for ServerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServerId({})", self.0)
    }
}

impl Display for ServerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ServerId {
    type Err = jid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ServerId(s.parse::<BareJid>()?))
    }
}

impl AsRef<BareJid> for ServerId {
    fn as_ref(&self) -> &BareJid {
        &self.0
    }
}

impl Borrow<Jid> for ServerId {
    fn borrow(&self) -> &Jid {
        &self.0
    }
}

impl From<ServerId> for BareJid {
    fn from(value: ServerId) -> Self {
        value.0
    }
}

impl IntoAttributeValue for ServerId {
    fn into_attribute_value(self) -> Option<String> {
        self.0.into_attribute_value()
    }
}

impl KeyType for ServerId {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.0.to_string())
    }
}
