// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::encryption::models::{
    DeviceBundle, DeviceId, IdentityKeyPair, PreKey, SignedPreKey,
};

#[derive(Clone, Debug)]
pub struct LocalEncryptionBundle {
    pub device_id: DeviceId,
    pub identity_key_pair: IdentityKeyPair,
    pub signed_pre_key: SignedPreKey,
    pub pre_keys: Vec<PreKey>,
}

impl LocalEncryptionBundle {
    pub fn into_device_bundle(self) -> DeviceBundle {
        DeviceBundle {
            device_id: self.device_id,
            signed_pre_key: self.signed_pre_key.into_public_signed_pre_key(),
            identity_key: self.identity_key_pair.identity_key,
            pre_keys: self
                .pre_keys
                .into_iter()
                .map(|key| key.into_public_pre_key())
                .collect(),
        }
    }
}
