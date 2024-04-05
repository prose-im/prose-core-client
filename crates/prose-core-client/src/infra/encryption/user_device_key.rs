// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use prose_store::{KeyType, RawKey};

use crate::domain::encryption::models::DeviceId;
use crate::dtos::UserId;

#[derive(Debug, Clone, PartialEq)]
pub struct UserDeviceKeyRef<'u, 'd> {
    user_id: &'u UserId,
    device_id: &'d DeviceId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SenderDistributionKeyRef<'u, 'de, 'di> {
    user_id: &'u UserId,
    device_id: &'de DeviceId,
    distribution_id: &'di Uuid,
}

impl<'u, 'd> UserDeviceKeyRef<'u, 'd> {
    pub fn new(user_id: &'u UserId, device_id: &'d DeviceId) -> Self {
        Self { user_id, device_id }
    }
}

impl KeyType for UserDeviceKeyRef<'_, '_> {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_string())
    }
}

impl<'u, 'de, 'di> SenderDistributionKeyRef<'u, 'de, 'di> {
    pub fn new(user_id: &'u UserId, device_id: &'de DeviceId, distribution_id: &'di Uuid) -> Self {
        Self {
            user_id,
            device_id,
            distribution_id,
        }
    }
}

impl Display for UserDeviceKeyRef<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.user_id, self.device_id.as_ref())
    }
}

impl KeyType for SenderDistributionKeyRef<'_, '_, '_> {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_string())
    }
}

impl Display for SenderDistributionKeyRef<'_, '_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            self.user_id,
            self.device_id.as_ref(),
            self.distribution_id
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserDeviceKey(String);

impl UserDeviceKey {
    pub fn min(user_id: &UserId) -> Self {
        Self::new(user_id, &DeviceId::from(u32::MIN))
    }

    pub fn max(user_id: &UserId) -> Self {
        Self::new(user_id, &DeviceId::from(u32::MAX))
    }

    pub fn parse_device_id_from_key(key: &str) -> Result<DeviceId> {
        let (_, device_id_str) = key
            .rsplit_once(".")
            .ok_or(anyhow!("Invalid UserDeviceKey"))?;
        Ok(DeviceId::from(device_id_str.parse::<u32>()?))
    }
}

impl UserDeviceKey {
    pub fn new(user_id: &UserId, device_id: &DeviceId) -> Self {
        Self(UserDeviceKeyRef::new(user_id, device_id).to_string())
    }
}

impl KeyType for UserDeviceKey {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.0.clone())
    }
}
