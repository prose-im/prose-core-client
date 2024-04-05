// prose-core-client/prose-xmpp
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;

use crate::{ns, ElementExt};

use super::Device;

#[derive(Debug, Clone, Default)]
pub struct DeviceList {
    pub devices: Vec<Device>,
}

impl TryFrom<Element> for DeviceList {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("list", ns::LEGACY_OMEMO)?;

        Ok(Self {
            devices: value
                .children()
                .map(|child| Device::try_from(child.clone()))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<DeviceList> for Element {
    fn from(value: DeviceList) -> Self {
        Element::builder("list", ns::LEGACY_OMEMO)
            .append_all(value.devices)
            .build()
    }
}
