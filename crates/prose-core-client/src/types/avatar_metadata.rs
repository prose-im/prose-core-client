use serde::{Deserialize, Serialize};

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
