use base64::{engine::general_purpose, Engine as _};
use sha1::{Digest, Sha1};

use prose_core_lib::modules::profile::avatar;
use prose_core_lib::modules::profile::avatar::ImageId;

use crate::types::error::StanzaParseError;

pub struct AvatarMetadata {
    pub mime_type: String,
    pub checksum: ImageId,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl AvatarMetadata {
    pub fn new(
        mime_type: impl Into<String>,
        checksum: ImageId,
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
    pub fn decode_base64_data(data: impl AsRef<[u8]>) -> anyhow::Result<Vec<u8>> {
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

impl<'a> TryFrom<avatar::Info<'a>> for AvatarMetadata {
    type Error = anyhow::Error;

    fn try_from(value: avatar::Info) -> Result<Self, Self::Error> {
        let Some(checksum) = value.id() else {
            return Err(anyhow::Error::new(
                StanzaParseError::missing_attribute("id", &value))
            )
        };
        let Some(mime_type) = value.r#type() else {
            return Err(anyhow::Error::new(
                StanzaParseError::missing_attribute("type", &value))
            )
        };

        Ok(AvatarMetadata {
            mime_type: mime_type.to_string(),
            checksum,
            width: value.width(),
            height: value.height(),
        })
    }
}
