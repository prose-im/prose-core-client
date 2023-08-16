// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
#[cfg(target_arch = "wasm32")]
use auto_impl::auto_impl;
use jid::BareJid;

use prose_xmpp::mods::AvatarData;
use prose_xmpp::stanza::avatar;
use prose_xmpp::{SendUnlessWasm, SyncUnlessWasm};

use crate::types::AvatarMetadata;

pub const MAX_IMAGE_DIMENSIONS: (u32, u32) = (400, 400);

#[cfg_attr(target_arch = "wasm32", async_trait(? Send), auto_impl(Rc))]
#[async_trait]
pub trait AvatarCache: SendUnlessWasm + SyncUnlessWasm {
    type Image;
    type Error: std::error::Error + Send + Sync;

    async fn cache_avatar_image(
        &self,
        jid: &BareJid,
        image: &AvatarData,
        metadata: &AvatarMetadata,
    ) -> Result<(), Self::Error>;

    async fn has_cached_avatar_image(
        &self,
        jid: &BareJid,
        image_checksum: &avatar::ImageId,
    ) -> Result<bool, Self::Error>;

    async fn cached_avatar_image(
        &self,
        jid: &BareJid,
        image_checksum: &avatar::ImageId,
    ) -> Result<Option<Self::Image>, Self::Error>;

    async fn delete_all_cached_images(&self) -> Result<(), Self::Error>;
}
