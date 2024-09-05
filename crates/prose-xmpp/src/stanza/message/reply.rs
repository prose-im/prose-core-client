// prose-core-client/prose-xmpp
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{ns, ElementExt, ParseError};
use jid::Jid;
use minidom::Element;
use xmpp_parsers::message::MessagePayload;

/// XEP-0461: Message Replies
/// https://xmpp.org/extensions/xep-0461.html
#[derive(Debug, PartialEq, Clone)]
pub struct Reply {
    pub id: String,
    pub to: Option<Jid>,
}

impl Reply {
    pub fn new(id: impl Into<String>, to: Option<impl Into<Jid>>) -> Self {
        Self {
            id: id.into(),
            to: to.map(Into::into),
        }
    }
}

impl TryFrom<Element> for Reply {
    type Error = ParseError;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("reply", ns::REPLY)?;

        Ok(Self {
            id: value.attr_req("id")?.to_string(),
            to: value
                .attr("to")
                .map(|attr| attr.parse())
                .transpose()
                .map_err(ParseError::from)?,
        })
    }
}

impl From<Reply> for Element {
    fn from(value: Reply) -> Self {
        Element::builder("reply", ns::REPLY)
            .attr("id", value.id)
            .attr("to", value.to)
            .build()
    }
}

impl MessagePayload for Reply {}
