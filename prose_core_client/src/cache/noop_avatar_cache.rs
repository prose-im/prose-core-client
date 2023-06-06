use std::path::PathBuf;

use image::DynamicImage;
use jid::BareJid;

use prose_core_lib::modules::profile::avatar::ImageId;

use crate::cache::AvatarCache;
use crate::types::AvatarMetadata;

pub struct NoopAvatarCache {}

impl NoopAvatarCache {
    pub fn new() -> Self {
        NoopAvatarCache {}
    }
}

impl AvatarCache for NoopAvatarCache {
    fn cache_avatar_image(
        &self,
        _jid: &BareJid,
        _image: DynamicImage,
        _metadata: &AvatarMetadata,
    ) -> anyhow::Result<PathBuf> {
        Ok(PathBuf::new())
    }

    fn cached_avatar_image_url(
        &self,
        _jid: &BareJid,
        _image_checksum: &ImageId,
    ) -> Option<PathBuf> {
        None
    }

    fn delete_all_cached_images(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
