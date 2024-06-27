// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Error;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::mods::AvatarData;

use crate::domain::shared::models::{AccountId, AvatarId, UserId};
use crate::domain::user_info::models::{AvatarInfo, PlatformImage};

pub const MAX_IMAGE_DIMENSIONS: (u32, u32) = (400, 400);

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
pub trait AvatarCache: SendUnlessWasm + SyncUnlessWasm {
    async fn cache_avatar_image(
        &self,
        account: &AccountId,
        user_id: &UserId,
        image: &AvatarData,
        metadata: &AvatarInfo,
    ) -> Result<(), Error>;

    async fn has_cached_avatar_image(
        &self,
        account: &AccountId,
        user_id: &UserId,
        image_checksum: &AvatarId,
    ) -> Result<bool, Error>;

    async fn cached_avatar_image(
        &self,
        account: &AccountId,
        user_id: &UserId,
        image_checksum: &AvatarId,
    ) -> Result<Option<PlatformImage>, Error>;

    async fn delete_all_cached_images(&self, account: &AccountId) -> Result<(), Error>;
}
