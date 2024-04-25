// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};

use crate::domain::shared::models::UserId;

use super::{DeviceId, IdentityKey, SessionData};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Trust {
    Undecided,
    Untrusted,
    Trusted,
    Verified,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: UserId,
    pub device_id: DeviceId,
    pub trust: Trust,
    pub is_active: bool,
    pub identity: Option<IdentityKey>,
    pub data: Option<SessionData>,
}

impl Session {
    pub fn is_trusted(&self) -> bool {
        match self.trust {
            Trust::Untrusted => false,
            Trust::Undecided => false,
            Trust::Trusted | Trust::Verified => true,
        }
    }

    pub fn is_trusted_or_undecided(&self) -> bool {
        self.is_trusted() || self.trust == Trust::Undecided
    }
}
