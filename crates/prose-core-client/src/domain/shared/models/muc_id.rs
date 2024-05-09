// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

use jid::{BareJid, Jid};
use minidom::IntoAttributeValue;
use serde::{Deserialize, Serialize};

use crate::dtos::OccupantId;
use crate::infra::xmpp::util::{JidExt, JidParseError};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Represents the BareJid of a MUC room.
pub struct MucId(BareJid);

impl MucId {
    pub fn occupant_id_with_nickname(
        &self,
        nickname: impl AsRef<str>,
    ) -> Result<OccupantId, jid::Error> {
        Ok(OccupantId::from(
            self.0.with_resource_str(nickname.as_ref())?,
        ))
    }

    pub fn into_inner(self) -> BareJid {
        self.0
    }
}

impl From<BareJid> for MucId {
    fn from(value: BareJid) -> Self {
        MucId(value)
    }
}

impl Debug for MucId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MucId({})", self.0)
    }
}

impl Display for MucId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<BareJid> for MucId {
    fn as_ref(&self) -> &BareJid {
        &self.0
    }
}

impl IntoAttributeValue for MucId {
    fn into_attribute_value(self) -> Option<String> {
        self.0.into_attribute_value()
    }
}

impl Deref for MucId {
    type Target = BareJid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for MucId {
    type Err = <BareJid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl From<MucId> for Jid {
    fn from(value: MucId) -> Self {
        Jid::Bare(value.0)
    }
}

impl MucId {
    pub fn from_iri(iri: &str) -> Result<Self, JidParseError> {
        Ok(Self(Jid::from_iri(iri)?.into_bare()))
    }
}
