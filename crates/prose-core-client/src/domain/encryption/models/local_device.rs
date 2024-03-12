// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::encryption::models::{DeviceId, IdentityKeyPair};

#[derive(Clone, Debug)]
pub struct LocalDevice {
    pub device_id: DeviceId,
    pub identity_key_pair: IdentityKeyPair,
}
