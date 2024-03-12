// prose-core-client/prose-xmpp
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;

use crate::{ns, ElementExt};

#[derive(Debug, Clone)]
pub struct Device {
    pub id: u32,
    pub label: Option<String>,
}

impl TryFrom<Element> for Device {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("device", ns::LEGACY_OMEMO)?;

        Ok(Self {
            id: value.attr_req("id")?.parse::<u32>()?.into(),
            label: value.attr("label").map(ToString::to_string),
        })
    }
}

impl From<Device> for Element {
    fn from(value: Device) -> Self {
        Element::builder("device", ns::LEGACY_OMEMO)
            .attr("id", value.id)
            .attr("label", value.label)
            .build()
    }
}
