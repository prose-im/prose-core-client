// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use minidom::IntoAttributeValue;
use serde::{Deserialize, Serialize};

use prose_store::{KeyType, RawKey};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeviceId(u32);

impl From<u32> for DeviceId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<DeviceId> for u32 {
    fn from(value: DeviceId) -> Self {
        value.0
    }
}

impl IntoAttributeValue for DeviceId {
    fn into_attribute_value(self) -> Option<String> {
        Some(self.0.to_string())
    }
}

impl FromStr for DeviceId {
    type Err = <u32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl AsRef<u32> for DeviceId {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl DeviceId {
    pub fn into_inner(self) -> u32 {
        self.0
    }
}

impl Display for DeviceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl KeyType for DeviceId {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Integer(self.0 as i64)
    }
}
