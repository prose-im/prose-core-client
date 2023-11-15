// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

use jid::BareJid;
use minidom::IntoAttributeValue;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A RoomJid while always a BareJid can either stand for a single contact or a MUC room.
pub struct RoomJid(BareJid);

impl RoomJid {
    pub fn into_inner(self) -> BareJid {
        self.0
    }
}

impl From<BareJid> for RoomJid {
    fn from(value: BareJid) -> Self {
        RoomJid(value)
    }
}

impl Deref for RoomJid {
    type Target = BareJid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for RoomJid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl IntoAttributeValue for RoomJid {
    fn into_attribute_value(self) -> Option<String> {
        self.0.into_attribute_value()
    }
}

impl FromStr for RoomJid {
    type Err = jid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(BareJid::from_str(s)?))
    }
}
