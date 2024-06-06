// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Display, Formatter};

use jid::BareJid;
use prose_store::{KeyType, RawKey};
use serde::{Deserialize, Serialize};

use crate::dtos::UserId;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Represents the BareJid of our logged-in user.
pub struct AccountId(UserId);

impl AccountId {
    pub fn into_user_id(self) -> UserId {
        UserId::from(self.0)
    }

    pub fn to_user_id(&self) -> UserId {
        UserId::from(self.0.clone())
    }
}

impl AccountId {
    pub fn is_same_domain(&self, other: &UserId) -> bool {
        self.0.is_same_domain(other)
    }
}

impl From<BareJid> for AccountId {
    fn from(value: BareJid) -> Self {
        AccountId(UserId::from(value))
    }
}

impl AsRef<UserId> for AccountId {
    fn as_ref(&self) -> &UserId {
        &self.0
    }
}

impl Debug for AccountId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AccountId({})", self.0)
    }
}

impl Display for AccountId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<UserId> for AccountId {
    fn eq(&self, other: &UserId) -> bool {
        &self.0 == other
    }
}

impl KeyType for AccountId {
    fn to_raw_key(&self) -> RawKey {
        self.0.to_raw_key()
    }
}
