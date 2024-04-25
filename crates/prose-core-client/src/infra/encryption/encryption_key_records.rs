// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};

use crate::domain::encryption::models::{
    DeviceId, IdentityKey, IdentityKeyPair, LocalDevice, Session, SessionData, Trust,
};
use crate::dtos::UserId;

pub mod collections {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionRecord {
    pub user_id: UserId,
    pub device_id: DeviceId,
    pub trust: Trust,
    pub is_active: bool,
    pub data: Option<SessionData>,
    pub identity: Option<IdentityKey>,
}

impl From<SessionRecord> for Session {
    fn from(value: SessionRecord) -> Self {
        Self {
            user_id: value.user_id,
            device_id: value.device_id,
            trust: value.trust,
            is_active: value.is_active,
            identity: value.identity,
            data: value.data,
        }
    }
}
