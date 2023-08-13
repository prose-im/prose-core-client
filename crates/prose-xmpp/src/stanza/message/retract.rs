// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::util::ElementExt;
use minidom::Element;

use crate::ns;
use crate::stanza::message::fasten;
use crate::stanza::message::fasten::ApplyTo;

pub struct Retract {}

impl Default for Retract {
    fn default() -> Self {
        Retract {}
    }
}

impl From<Retract> for Element {
    fn from(_value: Retract) -> Self {
        Element::builder("retract", ns::RETRACT).build()
    }
}

impl TryFrom<Element> for Retract {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("retract", ns::RETRACT)?;
        Ok(Retract::default())
    }
}

impl fasten::ApplyToPayload for Retract {}

impl ApplyTo {
    pub fn retract(&self) -> bool {
        self.payloads
            .iter()
            .find(|p| p.is("retract", ns::RETRACT))
            .is_some()
    }
}
