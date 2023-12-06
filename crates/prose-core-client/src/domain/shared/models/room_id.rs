// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{write, Debug, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

use jid::BareJid;
use minidom::IntoAttributeValue;

use super::OccupantId;

#[derive(Clone, PartialEq, Eq, Hash)]
/// A RoomJid while always a BareJid can either stand for a single contact or a MUC room.
pub struct RoomId(BareJid);

impl RoomId {
    pub fn into_inner(self) -> BareJid {
        self.0
    }

    pub fn occupant_id_with_nickname(
        &self,
        nickname: impl AsRef<str>,
    ) -> Result<OccupantId, jid::Error> {
        Ok(OccupantId::from(self.0.with_resource_str(nickname.as_ref())?))
    }
}

impl From<BareJid> for RoomId {
    fn from(value: BareJid) -> Self {
        RoomId(value)
    }
}

impl Deref for RoomId {
    type Target = BareJid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for RoomId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RoomId({})", self.0)
    }
}

impl Display for RoomId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl IntoAttributeValue for RoomId {
    fn into_attribute_value(self) -> Option<String> {
        self.0.into_attribute_value()
    }
}

impl FromStr for RoomId {
    type Err = jid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(BareJid::from_str(s)?))
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum RoomJidParseError {
    #[error("Missing xmpp: prefix in IRI")]
    InvalidIRI,
    #[error(transparent)]
    JID(#[from] jid::Error),
}

impl RoomId {
    pub fn from_iri(iri: &str) -> Result<Self, RoomJidParseError> {
        let Some(mut iri) = iri.strip_prefix("xmpp:") else {
            return Err(RoomJidParseError::InvalidIRI);
        };
        if let Some(idx) = iri.rfind("?join") {
            iri = &iri[..idx];
        }
        Ok(Self::from_str(iri)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::room_id;

    use super::*;

    #[test]
    fn test_from_iri() {
        assert!(RoomId::from_iri("").is_err());
        assert_eq!(
            RoomId::from_iri("xmpp:room@muc.example.org?join"),
            Ok(room_id!("room@muc.example.org"))
        );
        assert_eq!(
            RoomId::from_iri("xmpp:room@muc.example.org"),
            Ok(room_id!("room@muc.example.org"))
        );
    }
}
