// prose-core-client/prose-core-client
//
// Copyright: 2025, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{BareEntityId, OccupantId, ParticipantIdRef, ServerId, UserId};
use anyhow::anyhow;
use jid::Jid;
use prose_store::{KeyType, RawKey};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Represents any addressable XMPP entity with a unique identifier.
/// This can be a user (bare JID), a room occupant (full JID), or a server (bare JID).
/// Used for operations that can target any of these entity types, such as avatar storage.
pub enum EntityId {
    User(UserId),
    Occupant(OccupantId),
    Server(ServerId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum EntityIdRef<'a> {
    User(&'a UserId),
    Occupant(&'a OccupantId),
    Server(&'a ServerId),
}

impl Display for EntityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User(id) => write!(f, "{id}"),
            Self::Occupant(id) => write!(f, "{id}"),
            Self::Server(id) => write!(f, "{id}"),
        }
    }
}

impl Display for EntityIdRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User(id) => write!(f, "{id}"),
            Self::Occupant(id) => write!(f, "{id}"),
            Self::Server(id) => write!(f, "{id}"),
        }
    }
}

impl EntityId {
    pub fn to_ref(&self) -> EntityIdRef<'_> {
        match self {
            Self::User(id) => EntityIdRef::User(id),
            Self::Occupant(id) => EntityIdRef::Occupant(id),
            Self::Server(id) => EntityIdRef::Server(id),
        }
    }
}

impl<'a> EntityIdRef<'a> {
    pub fn to_owned(&self) -> EntityId {
        match *self {
            Self::User(id) => EntityId::User(id.clone()),
            Self::Occupant(id) => EntityId::Occupant(id.clone()),
            Self::Server(id) => EntityId::Server(id.clone()),
        }
    }

    pub fn to_raw_key_string(&self) -> String {
        match self {
            Self::User(id) => format!("user:{id}"),
            Self::Occupant(id) => format!("occ:{id}"),
            Self::Server(id) => format!("srv:{id}"),
        }
    }
}

impl FromStr for EntityId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            _ if s.starts_with("user:") => Ok(Self::User(s[5..].parse()?)),
            _ if s.starts_with("occ:") => Ok(Self::Occupant(s[4..].parse()?)),
            _ if s.starts_with("srv:") => Ok(Self::Server(s[4..].parse()?)),
            _ => Err(anyhow!("Scheme should be 'user', 'occ' or 'srv'")),
        }
    }
}

impl Serialize for EntityId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_ref().to_raw_key_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EntityId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(s.parse().map_err(serde::de::Error::custom)?)
    }
}

impl KeyType for EntityId {
    fn to_raw_key(&self) -> RawKey {
        self.to_ref().to_raw_key()
    }
}

impl KeyType for EntityIdRef<'_> {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_raw_key_string())
    }
}

impl<'a> From<&'a UserId> for EntityIdRef<'a> {
    fn from(value: &'a UserId) -> Self {
        Self::User(value)
    }
}

impl<'a> From<&'a OccupantId> for EntityIdRef<'a> {
    fn from(value: &'a OccupantId) -> Self {
        Self::Occupant(value)
    }
}

impl<'a> From<&'a ServerId> for EntityIdRef<'a> {
    fn from(value: &'a ServerId) -> Self {
        Self::Server(value)
    }
}

impl<'a> From<&'a BareEntityId> for EntityIdRef<'a> {
    fn from(value: &'a BareEntityId) -> Self {
        match value {
            BareEntityId::User(id) => Self::User(id),
            BareEntityId::Server(id) => Self::Server(id),
        }
    }
}

impl<'a> From<ParticipantIdRef<'a>> for EntityIdRef<'a> {
    fn from(value: ParticipantIdRef<'a>) -> Self {
        match value {
            ParticipantIdRef::Occupant(id) => Self::Occupant(id),
            ParticipantIdRef::User(id) => Self::User(id),
        }
    }
}

impl Borrow<Jid> for EntityIdRef<'_> {
    fn borrow(&self) -> &Jid {
        match *self {
            Self::User(id) => id.borrow(),
            Self::Occupant(id) => id.borrow(),
            Self::Server(id) => id.borrow(),
        }
    }
}

impl AsRef<Jid> for EntityIdRef<'_> {
    fn as_ref(&self) -> &Jid {
        self.borrow()
    }
}
