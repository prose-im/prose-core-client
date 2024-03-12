// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

use super::DeviceId;

#[derive(Debug, Clone, PartialEq)]
pub struct Device {
    pub id: DeviceId,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceList {
    pub devices: Vec<Device>,
}

impl Display for Device {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(label) = &self.label {
            write!(f, "{} (\"{label}\")", self.id.as_ref())
        } else {
            write!(f, "{}", self.id.as_ref())
        }
    }
}
