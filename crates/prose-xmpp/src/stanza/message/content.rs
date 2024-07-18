// prose-core-client/prose-xmpp
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;
use xmpp_parsers::message::MessagePayload;

use crate::{ns, ElementExt, ParseError};

/// XEP-0481: Content Types in Messages
pub struct Content {
    pub r#type: String,
    pub content: String,
}

impl From<Content> for Element {
    fn from(value: Content) -> Self {
        Element::builder("content", ns::CONTENT)
            .attr("type", value.r#type)
            .append(value.content)
            .build()
    }
}

impl TryFrom<Element> for Content {
    type Error = ParseError;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("content", ns::CONTENT)?;

        Ok(Content {
            r#type: value.attr_req("type")?.to_string(),
            content: value.text(),
        })
    }
}

impl MessagePayload for Content {}
