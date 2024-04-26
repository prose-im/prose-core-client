// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{DeviceId, IdentityKey, Trust};

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceInfo {
    pub id: DeviceId,
    pub identity: IdentityKey,
    pub trust: Trust,
    pub is_active: bool,
    pub is_this_device: bool,
}

impl DeviceInfo {
    pub fn fingerprint(&self) -> String {
        self.identity.fingerprint()
    }
}
