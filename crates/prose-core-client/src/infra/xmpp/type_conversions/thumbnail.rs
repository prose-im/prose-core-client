// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use mime::Mime;

pub use prose_xmpp::stanza::media_sharing::Thumbnail as XMPPThumbnail;

use crate::domain::messaging::models::Thumbnail;

impl TryFrom<XMPPThumbnail> for Thumbnail {
    type Error = anyhow::Error;

    fn try_from(value: XMPPThumbnail) -> Result<Self, Self::Error> {
        Ok(Thumbnail {
            url: value.uri.parse()?,
            media_type: value
                .media_type
                .map(|mt| mt.parse::<Mime>())
                .transpose()?
                .unwrap_or(mime::APPLICATION_OCTET_STREAM),
            width: value.width,
            height: value.height,
        })
    }
}

impl From<Thumbnail> for XMPPThumbnail {
    fn from(value: Thumbnail) -> Self {
        XMPPThumbnail {
            uri: value.url.to_string(),
            media_type: Some(value.media_type.to_string()),
            width: value.width,
            height: value.height,
        }
    }
}
