use std::path::PathBuf;

use anyhow::Result;
use jid::BareJid;

use prose_xmpp::stanza::avatar;

use crate::avatar_cache::AvatarCache;

pub struct NoopAvatarCache {}

impl Default for NoopAvatarCache {
    fn default() -> Self {
        NoopAvatarCache {}
    }
}

impl AvatarCache for NoopAvatarCache {
    #[cfg(feature = "native-app")]
    fn cache_avatar_image(
        &self,
        _jid: &BareJid,
        _image: image::DynamicImage,
        _metadata: &crate::types::AvatarMetadata,
    ) -> Result<PathBuf> {
        Ok(PathBuf::new())
    }

    fn cached_avatar_image_url(
        &self,
        _jid: &BareJid,
        _image_checksum: &avatar::ImageId,
    ) -> Option<PathBuf> {
        None
    }

    fn delete_all_cached_images(&self) -> Result<()> {
        Ok(())
    }
}
