// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use jid::BareJid;
use thiserror::Error;

use prose_xmpp::mods::AvatarData;
use prose_xmpp::stanza::avatar;
use prose_xmpp::stanza::avatar::ImageId;

use crate::avatar_cache::AvatarCache;
use crate::types::AvatarMetadata;

#[derive(Default)]
pub struct NoopAvatarCache {}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct NoopAvatarCacheError(#[from] anyhow::Error);

type Result<T> = std::result::Result<T, NoopAvatarCacheError>;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl AvatarCache for NoopAvatarCache {
    type Image = ();
    type Error = NoopAvatarCacheError;

    async fn cache_avatar_image(
        &self,
        _jid: &BareJid,
        _image: &AvatarData,
        _metadata: &AvatarMetadata,
    ) -> Result<()> {
        Ok(())
    }

    async fn has_cached_avatar_image(
        &self,
        _jid: &BareJid,
        _image_checksum: &ImageId,
    ) -> Result<bool> {
        Ok(false)
    }

    async fn cached_avatar_image(
        &self,
        _jid: &BareJid,
        _image_checksum: &avatar::ImageId,
    ) -> Result<Option<Self::Image>> {
        Ok(None)
    }

    async fn delete_all_cached_images(&self) -> Result<()> {
        Ok(())
    }
}
