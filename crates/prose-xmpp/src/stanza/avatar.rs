// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;
pub use xmpp_parsers::avatar::Data;
use xmpp_parsers::pubsub::PubSubPayload;

use prose_utils::id_string;

use crate::util::ElementExt;
use crate::{ns, ParseError};

id_string!(ImageId);

#[derive(Debug, PartialEq, Clone)]
pub struct Metadata {
    pub infos: Vec<Info>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Info {
    pub bytes: u32,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub id: ImageId,
    pub r#type: String,
    pub url: Option<String>,
}

impl TryFrom<Element> for Metadata {
    type Error = ParseError;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("metadata", ns::AVATAR_METADATA)?;

        Ok(Metadata {
            infos: value
                .children()
                .map(|c| c.clone())
                .map(Info::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl From<Metadata> for Element {
    fn from(value: Metadata) -> Self {
        Element::builder("metadata", ns::AVATAR_METADATA)
            .append_all(value.infos)
            .build()
    }
}

impl PubSubPayload for Metadata {}

impl TryFrom<Element> for Info {
    type Error = ParseError;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("info", ns::AVATAR_METADATA)?;

        Ok(Info {
            bytes: value.attr_req("bytes")?.parse()?,
            width: value.attr("width").map(|w| w.parse()).transpose()?,
            height: value.attr("height").map(|h| h.parse()).transpose()?,
            id: value.attr_req("id")?.into(),
            r#type: value.attr_req("type")?.to_string(),
            url: value.attr("url").map(ToString::to_string),
        })
    }
}

impl From<Info> for Element {
    fn from(value: Info) -> Self {
        Element::builder("info", ns::AVATAR_METADATA)
            .attr("bytes", value.bytes)
            .attr("width", value.width)
            .attr("height", value.height)
            .attr("id", value.id)
            .attr("type", value.r#type)
            .attr("url", value.url)
            .build()
    }
}
