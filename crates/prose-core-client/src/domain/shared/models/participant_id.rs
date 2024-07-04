// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use anyhow::anyhow;
use jid::Jid;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use prose_store::{KeyType, RawKey};

use crate::dtos::RoomId;

use super::{OccupantId, UserEndpointId, UserId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Represents the identifier of a user within - what we define as - room. So it could be either a
/// regular UserId (BareJid) in a DirectMessage room (1:1 conversation) or a OccupantId when in a
/// multi-user room (MUC chat).
pub enum ParticipantId {
    User(UserId),
    Occupant(OccupantId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum ParticipantIdRef<'a> {
    User(&'a UserId),
    Occupant(&'a OccupantId),
}

impl Display for ParticipantId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParticipantId::User(id) => write!(f, "{id}"),
            ParticipantId::Occupant(id) => write!(f, "{id}"),
        }
    }
}

impl Display for ParticipantIdRef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParticipantIdRef::User(id) => write!(f, "{id}"),
            ParticipantIdRef::Occupant(id) => write!(f, "{id}"),
        }
    }
}

impl ParticipantId {
    pub fn to_user_id(&self) -> Option<UserId> {
        let ParticipantId::User(id) = &self else {
            return None;
        };
        Some(id.clone())
    }

    pub fn to_occupant_id(&self) -> Option<OccupantId> {
        let ParticipantId::Occupant(id) = &self else {
            return None;
        };
        Some(id.clone())
    }

    pub fn to_room_id(&self) -> RoomId {
        match self {
            ParticipantId::User(id) => RoomId::User(id.clone()),
            ParticipantId::Occupant(id) => RoomId::Muc(id.muc_id()),
        }
    }

    pub fn to_opaque_identifier(&self) -> String {
        match self {
            ParticipantId::User(id) => id.to_string(),
            ParticipantId::Occupant(id) => id.to_string(),
        }
    }

    pub fn to_ref(&self) -> ParticipantIdRef<'_> {
        match self {
            ParticipantId::User(id) => ParticipantIdRef::User(id),
            ParticipantId::Occupant(id) => ParticipantIdRef::Occupant(id),
        }
    }
}

impl<'a> ParticipantIdRef<'a> {
    pub fn to_owned(&self) -> ParticipantId {
        match *self {
            ParticipantIdRef::User(id) => ParticipantId::User(id.clone()),
            ParticipantIdRef::Occupant(id) => ParticipantId::Occupant(id.clone()),
        }
    }

    pub fn to_user_id(&self) -> Option<&UserId> {
        let Self::User(id) = &self else {
            return None;
        };
        Some(id)
    }

    pub fn to_occupant_id(&self) -> Option<&OccupantId> {
        let Self::Occupant(id) = &self else {
            return None;
        };
        Some(id)
    }

    pub fn to_raw_key_string(&self) -> String {
        match self {
            Self::User(id) => format!("user:{id}"),
            Self::Occupant(id) => format!("occ:{id}"),
        }
    }
}

impl FromStr for ParticipantId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            _ if s.starts_with("user:") => Ok(ParticipantId::User(s[5..].parse()?)),
            _ if s.starts_with("occ:") => Ok(ParticipantId::Occupant(s[4..].parse()?)),
            _ => Err(anyhow!("Scheme should be 'user' or 'occ'")),
        }
    }
}

impl Serialize for ParticipantId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_ref().to_raw_key_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ParticipantId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(s.parse().map_err(serde::de::Error::custom)?)
    }
}

impl From<UserId> for ParticipantId {
    fn from(value: UserId) -> Self {
        ParticipantId::User(value)
    }
}

impl From<OccupantId> for ParticipantId {
    fn from(value: OccupantId) -> Self {
        ParticipantId::Occupant(value)
    }
}

impl From<UserEndpointId> for ParticipantId {
    fn from(value: UserEndpointId) -> Self {
        match value {
            UserEndpointId::User(id) => id.into(),
            UserEndpointId::UserResource(id) => id.into_user_id().into(),
            UserEndpointId::Occupant(id) => id.into(),
        }
    }
}

impl From<ParticipantId> for Jid {
    fn from(value: ParticipantId) -> Self {
        match value {
            ParticipantId::User(id) => Jid::from(id.into_inner()),
            ParticipantId::Occupant(id) => Jid::from(id.into_inner()),
        }
    }
}

impl KeyType for ParticipantId {
    fn to_raw_key(&self) -> RawKey {
        self.to_ref().to_raw_key()
    }
}

impl KeyType for ParticipantIdRef<'_> {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_raw_key_string())
    }
}

impl<'a> From<&'a UserId> for ParticipantIdRef<'a> {
    fn from(value: &'a UserId) -> Self {
        Self::User(value)
    }
}

impl<'a> From<&'a OccupantId> for ParticipantIdRef<'a> {
    fn from(value: &'a OccupantId) -> Self {
        Self::Occupant(value)
    }
}

impl Borrow<Jid> for ParticipantIdRef<'_> {
    fn borrow(&self) -> &Jid {
        match *self {
            Self::User(id) => id.borrow(),
            Self::Occupant(id) => id.borrow(),
        }
    }
}

impl AsRef<Jid> for ParticipantIdRef<'_> {
    fn as_ref(&self) -> &Jid {
        self.borrow()
    }
}
