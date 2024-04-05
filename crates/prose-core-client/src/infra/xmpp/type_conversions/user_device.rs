// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;

use prose_xmpp::stanza::omemo::{Device as XMPPDevice, DeviceList as XMPPDeviceList};

use crate::domain::encryption::models::{Device, DeviceId, DeviceList};

impl TryFrom<Element> for DeviceList {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        Ok(Self::from(XMPPDeviceList::try_from(value)?))
    }
}

impl From<XMPPDevice> for Device {
    fn from(value: XMPPDevice) -> Self {
        Self {
            id: DeviceId::from(value.id),
            label: value.label,
        }
    }
}

impl From<Device> for XMPPDevice {
    fn from(value: Device) -> Self {
        Self {
            id: *value.id.as_ref(),
            label: value.label,
        }
    }
}

impl From<XMPPDeviceList> for DeviceList {
    fn from(value: XMPPDeviceList) -> Self {
        Self {
            devices: value.devices.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<DeviceList> for XMPPDeviceList {
    fn from(value: DeviceList) -> Self {
        Self {
            devices: value.devices.into_iter().map(Into::into).collect(),
        }
    }
}
