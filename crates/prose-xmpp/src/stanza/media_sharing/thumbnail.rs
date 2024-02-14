// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;

use crate::{ns, ElementExt};

/// XEP-0264: Jingle Content Thumbnails
/// https://xmpp.org/extensions/xep-0264.html
#[derive(Debug, Clone, PartialEq)]
pub struct Thumbnail {
    pub uri: String,
    pub media_type: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl TryFrom<Element> for Thumbnail {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("thumbnail", ns::JINGLE_THUMBS)?;

        Ok(Self {
            uri: value.attr_req("uri")?.to_string(),
            media_type: value.attr("media-type").map(ToString::to_string),
            width: value.attr("width").map(|w| w.parse()).transpose()?,
            height: value.attr("height").map(|h| h.parse()).transpose()?,
        })
    }
}

impl From<Thumbnail> for Element {
    fn from(value: Thumbnail) -> Self {
        Element::builder("thumbnail", ns::JINGLE_THUMBS)
            .attr("uri", value.uri)
            .attr("media-type", value.media_type)
            .attr("width", value.width)
            .attr("height", value.height)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use anyhow::Result;

    use super::*;

    #[test]
    fn test_deserialize_thumbnail() -> Result<()> {
        assert_eq!(
            Thumbnail::try_from(Element::from_str(
                r#"<thumbnail xmlns='urn:xmpp:thumbs:1' uri='cid:sha1+ffd7c8d28e9c5e82afea41f97108c6b4@bob.xmpp.org' media-type='image/png' width='128' height='96'/>"#
            )?)?,
            Thumbnail {
                uri: "cid:sha1+ffd7c8d28e9c5e82afea41f97108c6b4@bob.xmpp.org".to_string(),
                media_type: Some("image/png".to_string()),
                width: Some(128),
                height: Some(96),
            }
        );

        assert_eq!(
            Thumbnail::try_from(Element::from_str(
                r#"<thumbnail xmlns='urn:xmpp:thumbs:1' uri='cid:sha1+ffd7c8d28e9c5e82afea41f97108c6b4@bob.xmpp.org'/>"#
            )?)?,
            Thumbnail {
                uri: "cid:sha1+ffd7c8d28e9c5e82afea41f97108c6b4@bob.xmpp.org".to_string(),
                media_type: None,
                width: None,
                height: None,
            }
        );

        Ok(())
    }

    #[test]
    fn test_serialize_thumbnail() -> Result<()> {
        let thumbnail = Thumbnail {
            uri: "cid:sha1+ffd7c8d28e9c5e82afea41f97108c6b4@bob.xmpp.org".to_string(),
            media_type: Some("image/png".to_string()),
            width: Some(128),
            height: Some(96),
        };

        let elem = Element::try_from(thumbnail.clone())?;
        let parsed_thumbnail = Thumbnail::try_from(elem)?;

        assert_eq!(thumbnail, parsed_thumbnail);

        Ok(())
    }
}
