// prose-core-client/prose-core-client
//
// Copyright: 2025, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{ServerId, UserId};
use anyhow::anyhow;
use jid::BareJid;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Represents a bare JID entity - either a user (with localpart@domain) or a server (domain only).
pub enum BareEntityId {
    User(UserId),
    Server(ServerId),
}

impl BareEntityId {
    pub fn into_inner(self) -> BareJid {
        match self {
            BareEntityId::User(user) => user.into_inner(),
            BareEntityId::Server(server) => server.into_inner(),
        }
    }
}

impl Display for BareEntityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User(id) => write!(f, "{id}"),
            Self::Server(id) => write!(f, "{id}"),
        }
    }
}

impl From<UserId> for BareEntityId {
    fn from(user_id: UserId) -> Self {
        Self::User(user_id)
    }
}

impl From<ServerId> for BareEntityId {
    fn from(server_id: ServerId) -> Self {
        Self::Server(server_id)
    }
}

impl From<BareJid> for BareEntityId {
    fn from(value: BareJid) -> Self {
        if value.node().is_some() {
            Self::User(value.into())
        } else {
            Self::Server(value.into())
        }
    }
}

impl BareEntityId {
    pub fn to_raw_key_string(&self) -> String {
        match self {
            Self::User(id) => format!("user:{id}"),
            Self::Server(id) => format!("srv:{id}"),
        }
    }
}

impl FromStr for BareEntityId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            _ if s.starts_with("user:") => Ok(Self::User(s[5..].parse()?)),
            _ if s.starts_with("srv:") => Ok(Self::Server(s[4..].parse()?)),
            _ => Err(anyhow!("Scheme should be 'user' or 'srv'")),
        }
    }
}

impl Serialize for BareEntityId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_raw_key_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BareEntityId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(s.parse().map_err(serde::de::Error::custom)?)
    }
}
