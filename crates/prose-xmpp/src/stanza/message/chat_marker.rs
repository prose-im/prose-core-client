// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;
use xmpp_parsers::message::MessagePayload;

use crate::ns;
use crate::stanza::message;
use crate::util::ElementExt;

// https://xmpp.org/extensions/xep-0333.html

// TODO: id can either be a MessageId or a StanzaId
// Therefore, if a MUC announces support for Unique and Stable Stanza IDs (XEP-0359) [9] then
// clients MUST always use the MUC-assigned id for Chat Markers. The id will be contained in a
// <stanza-id/> element inserted into the stanza with a 'by' attribute matching the MUC's own JID.

#[derive(Debug, PartialEq, Clone)]
pub struct Markable {}

#[derive(Debug, PartialEq, Clone)]
pub struct Received {
    pub id: message::Id,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Displayed {
    pub id: message::Id,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Acknowledged {
    pub id: message::Id,
}

impl TryFrom<Element> for Markable {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("markable", ns::CHAT_MARKERS)?;
        Ok(Markable {})
    }
}

impl From<Markable> for Element {
    fn from(_value: Markable) -> Self {
        Element::builder("markable", ns::CHAT_MARKERS).build()
    }
}

impl TryFrom<Element> for Received {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("received", ns::CHAT_MARKERS)?;
        Ok(Received {
            id: value.attr_req("id")?.into(),
        })
    }
}

impl From<Received> for Element {
    fn from(value: Received) -> Self {
        Element::builder("received", ns::CHAT_MARKERS)
            .attr("id", value.id)
            .build()
    }
}

impl TryFrom<Element> for Displayed {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("displayed", ns::CHAT_MARKERS)?;
        Ok(Displayed {
            id: value.attr_req("id")?.into(),
        })
    }
}

impl From<Displayed> for Element {
    fn from(value: Displayed) -> Self {
        Element::builder("displayed", ns::CHAT_MARKERS)
            .attr("id", value.id)
            .build()
    }
}

impl TryFrom<Element> for Acknowledged {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("acknowledged", ns::CHAT_MARKERS)?;
        Ok(Acknowledged {
            id: value.attr_req("id")?.into(),
        })
    }
}

impl From<Acknowledged> for Element {
    fn from(value: Acknowledged) -> Self {
        Element::builder("acknowledged", ns::CHAT_MARKERS)
            .attr("id", value.id)
            .build()
    }
}

impl MessagePayload for Markable {}
impl MessagePayload for Received {}
impl MessagePayload for Displayed {}
impl MessagePayload for Acknowledged {}
