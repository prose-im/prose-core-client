// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use jid::Jid;
use minidom::Element;
use xmpp_parsers::message::MessagePayload;

use prose_utils::id_string;

use crate::ns;
use crate::util::ElementExt;

// XEP-0359: Unique and Stable Stanza ID

id_string!(Id);

#[derive(Debug, PartialEq, Clone)]
pub struct OriginId {
    pub id: Id,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StanzaId {
    pub id: Id,
    pub by: Jid,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReferencedStanza {
    pub id: Id,
    pub by: Jid,
}

impl TryFrom<Element> for OriginId {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("origin-id", ns::SID)?;

        Ok(OriginId {
            id: value.attr_req("id")?.into(),
        })
    }
}

impl From<OriginId> for Element {
    fn from(value: OriginId) -> Self {
        Element::builder("origin-id", ns::SID)
            .attr("id", value.id)
            .build()
    }
}

impl TryFrom<Element> for StanzaId {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("stanza-id", ns::SID)?;

        Ok(StanzaId {
            id: value.attr_req("id")?.into(),
            by: Jid::from_str(value.attr_req("by")?)?,
        })
    }
}

impl From<StanzaId> for Element {
    fn from(value: StanzaId) -> Self {
        Element::builder("stanza-id", ns::SID)
            .attr("id", value.id)
            .attr("by", value.by)
            .build()
    }
}

impl TryFrom<Element> for ReferencedStanza {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("referenced-stanza", ns::SID)?;

        Ok(ReferencedStanza {
            id: value.attr_req("id")?.into(),
            by: Jid::from_str(value.attr_req("by")?)?,
        })
    }
}

impl From<ReferencedStanza> for Element {
    fn from(value: ReferencedStanza) -> Self {
        Element::builder("referenced-stanza", ns::SID)
            .attr("id", value.id)
            .attr("by", value.by)
            .build()
    }
}

impl MessagePayload for StanzaId {}
