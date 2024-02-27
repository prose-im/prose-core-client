// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Display, Formatter};

use jid::BareJid;

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

impl KeyType for RoomId {
    fn to_raw_key(&self) -> RawKey {
        (&self).to_raw_key()
    }
}

impl KeyType for &RoomId {
    fn to_raw_key(&self) -> RawKey {
        match self {
            RoomId::User(id) => RawKey::Text(id.to_string()),
            RoomId::Muc(id) => RawKey::Text(id.to_string()),
        }
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
