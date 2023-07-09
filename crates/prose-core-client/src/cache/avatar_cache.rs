use std::path::PathBuf;

use anyhow::Result;
use jid::BareJid;

use prose_xmpp::stanza::avatar;

pub const MAX_IMAGE_DIMENSIONS: (u32, u32) = (600, 600);
pub const IMAGE_OUTPUT_MIME_TYPE: &str = "image/jpeg";

pub trait AvatarCache: Send + Sync {
    #[cfg(feature = "native-app")]
    fn cache_avatar_image(
        &self,
        jid: &BareJid,
        image: image::DynamicImage,
        metadata: &crate::types::AvatarMetadata,
    ) -> Result<PathBuf>;

    fn cached_avatar_image_url(
        &self,
        jid: &BareJid,
        image_checksum: &avatar::ImageId,
    ) -> Option<PathBuf>;

    fn delete_all_cached_images(&self) -> Result<()>;
}
