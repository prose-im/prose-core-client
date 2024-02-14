// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use mime::Mime;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::util::mime_serde_shim;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attachment {
    pub r#type: AttachmentType,
    pub url: Url,
    #[serde(with = "mime_serde_shim")]
    pub media_type: Mime,
    pub file_name: String,
    pub file_size: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttachmentType {
    Audio {
        duration: Option<u64>,
    },
    Image {
        thumbnail: Option<Thumbnail>,
    },
    Video {
        duration: Option<u64>,
        thumbnail: Option<Thumbnail>,
    },
    File,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Thumbnail {
    pub url: Url,
    #[serde(with = "mime_serde_shim")]
    pub media_type: Mime,
    pub width: Option<u32>,
    pub height: Option<u32>,
}
