// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};

use crate::domain::encryption::models::{DeviceId, IdentityKeyPair, LocalDevice};

pub mod collections {
    pub const IDENTITY: &str = "omemo_identity";
    pub const KYBER_PRE_KEY: &str = "omemo_kyber_pre_key";
    pub const LOCAL_DEVICE: &str = "omemo_local_device";
    pub const PRE_KEY: &str = "omemo_pre_key";
    pub const SENDER_KEY: &str = "omemo_sender_key";
    pub const SESSION_RECORD: &str = "omemo_session_record";
    pub const SIGNED_PRE_KEY: &str = "omemo_signed_pre_key";
}

#[derive(Serialize, Deserialize)]
pub struct LocalDeviceRecord {
    pub device_id: DeviceId,
    pub identity_key_pair: IdentityKeyPair,
}

impl LocalDeviceRecord {
    pub fn current_id() -> &'static str {
        "current"
    }
}

impl From<LocalDeviceRecord> for LocalDevice {
    fn from(value: LocalDeviceRecord) -> Self {
        Self {
            device_id: value.device_id,
            identity_key_pair: value.identity_key_pair,
        }
    }
}
