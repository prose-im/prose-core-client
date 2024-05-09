// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Display, Formatter};

use jid::BareJid;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use prose_store::{KeyType, RawKey};

use crate::dtos::UserId;

use super::MucId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A RoomJid while always a BareJid can either stand for a single contact or a MUC room.
pub enum RoomId {
    User(UserId),
    Muc(MucId),
}

impl RoomId {
    pub fn muc_id(&self) -> Option<&MucId> {
        match self {
            RoomId::User(_) => None,
            RoomId::Muc(id) => Some(id),
        }
    }

    pub fn user_id(&self) -> Option<&UserId> {
        match self {
            RoomId::User(id) => Some(id),
            RoomId::Muc(_) => None,
        }
    }

    pub fn is_muc_room(&self) -> bool {
        match self {
            RoomId::User(_) => false,
            RoomId::Muc(_) => true,
        }
    }
}

impl From<UserId> for RoomId {
    fn from(value: UserId) -> Self {
        RoomId::User(value)
    }
}

impl From<MucId> for RoomId {
    fn from(value: MucId) -> Self {
        RoomId::Muc(value)
    }
}

impl Display for RoomId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RoomId::User(id) => write!(f, "{}", id),
            RoomId::Muc(id) => write!(f, "{}", id),
        }
    }
}

#[cfg(feature = "test")]
impl RoomId {
    pub fn to_display_name(&self) -> String {
        use crate::util::StringExt;

        let Some(node) = self.as_ref().node_str() else {
            return self.to_string().to_uppercase_first_letter();
        };
        node.capitalized_display_name()
    }
}

impl RoomId {
    pub fn to_raw_key_string(&self) -> String {
        match self {
            RoomId::User(id) => format!("user:{id}"),
            RoomId::Muc(id) => format!("muc:{id}"),
        }
    }
}

impl KeyType for RoomId {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_raw_key_string())
    }
}

impl RoomId {
    pub fn into_bare(self) -> BareJid {
        match self {
            RoomId::User(id) => id.into_inner(),
            RoomId::Muc(id) => id.into_inner(),
        }
    }
}

impl AsRef<BareJid> for RoomId {
    fn as_ref(&self) -> &BareJid {
        match self {
            RoomId::User(id) => id.as_ref(),
            RoomId::Muc(id) => id.as_ref(),
        }
    }
}

impl Serialize for RoomId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_raw_key_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RoomId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;

        match s {
            _ if s.starts_with("user:") => Ok(RoomId::User(
                s[5..].parse().map_err(serde::de::Error::custom)?,
            )),
            _ if s.starts_with("muc:") => Ok(RoomId::Muc(
                s[4..].parse().map_err(serde::de::Error::custom)?,
            )),
            _ => Err(serde::de::Error::custom("Scheme should be 'user' or 'muc'")),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{muc_id, user_id};

    use super::*;

    #[test]
    fn test_serializes_to_json() -> Result<()> {
        let room_id_str = r#""user:hello@prose.org""#;
        let room_id = RoomId::User(user_id!("hello@prose.org"));

        assert_eq!(room_id, serde_json::from_str(room_id_str)?);
        assert_eq!(room_id_str, &serde_json::to_string(&room_id)?);
        assert_eq!(
            RawKey::Text("user:hello@prose.org".to_string()),
            room_id.to_raw_key()
        );

        let room_id_str = r#""muc:room@conf.prose.org""#;
        let room_id = RoomId::Muc(muc_id!("room@conf.prose.org"));

        assert_eq!(room_id, serde_json::from_str(room_id_str)?);
        assert_eq!(room_id_str, &serde_json::to_string(&room_id)?);

        Ok(())
    }
}
