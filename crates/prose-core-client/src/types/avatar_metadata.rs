use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use prose_xmpp::stanza::avatar;

#[derive(Serialize, Deserialize)]
pub struct AvatarMetadata {
    pub mime_type: String,
    pub checksum: avatar::ImageId,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl AvatarMetadata {
    pub fn new(
        mime_type: impl Into<String>,
        checksum: avatar::ImageId,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Self {
        AvatarMetadata {
            mime_type: mime_type.into(),
            checksum: checksum.into(),
            width,
            height,
        }
    }
}

impl AvatarMetadata {
    pub fn decode_base64_data(data: impl AsRef<[u8]>) -> Result<Vec<u8>> {
        Ok(general_purpose::STANDARD.decode(data)?)
    }

    pub fn encode_image_data(data: impl AsRef<[u8]>) -> String {
        general_purpose::STANDARD.encode(data)
    }

    pub fn generate_sha1_checksum(data: impl AsRef<[u8]>) -> String {
        let mut hasher = Sha1::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
}

impl AvatarMetadata {}

impl From<avatar::Info> for AvatarMetadata {
    fn from(value: avatar::Info) -> Self {
        AvatarMetadata {
            mime_type: value.r#type,
            checksum: value.id,
            width: value.width.map(u32::from),
            height: value.height.map(u32::from),
        }
    }
}
