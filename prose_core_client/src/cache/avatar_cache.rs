use std::path::PathBuf;

use image::{DynamicImage, ImageOutputFormat};
use jid::BareJid;
use prose_core_lib::modules::profile::avatar::ImageId;

use crate::types::AvatarMetadata;

pub const MAX_IMAGE_DIMENSIONS: (u32, u32) = (600, 600);
pub const IMAGE_OUTPUT_FORMAT: ImageOutputFormat = ImageOutputFormat::Jpeg(80);
pub const IMAGE_OUTPUT_MIME_TYPE: &str = "image/jpeg";

pub trait AvatarCache: Send + Sync {
    fn cache_avatar_image(
        &self,
        jid: &BareJid,
        image: DynamicImage,
        metadata: &AvatarMetadata,
    ) -> anyhow::Result<PathBuf>;
    fn cached_avatar_image_url(&self, jid: &BareJid, image_checksum: &ImageId) -> Option<PathBuf>;

    fn delete_all_cached_images(&self) -> anyhow::Result<()>;
}
