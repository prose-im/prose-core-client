use minidom::Element;
pub use xmpp_parsers::avatar::Data;
use xmpp_parsers::pubsub::PubSubPayload;

use crate::ns;
use crate::util::id_string_macro::id_string;
use crate::util::ElementExt;

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
    type Error = anyhow::Error;

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
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("info", ns::AVATAR_METADATA)?;

        Ok(Info {
            bytes: value.req_attr("bytes")?.parse()?,
            width: value.attr("width").map(|w| w.parse()).transpose()?,
            height: value.attr("height").map(|h| h.parse()).transpose()?,
            id: value.req_attr("id")?.into(),
            r#type: value.req_attr("type")?.to_string(),
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