// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::encryption::models::{DeviceId, IdentityKey, PublicPreKey, PublicSignedPreKey};

#[derive(Clone, Debug)]
pub struct DeviceBundle {
    pub device_id: DeviceId,
    pub signed_pre_key: PublicSignedPreKey,
    pub identity_key: IdentityKey,
    pub pre_keys: Vec<PublicPreKey>,
}

#[derive(Clone, Debug)]
pub struct PreKeyBundle {
    pub device_id: DeviceId,
    pub signed_pre_key: PublicSignedPreKey,
    pub identity_key: IdentityKey,
    pub pre_key: PublicPreKey,
}
