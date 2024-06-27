// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::mods::AvatarData;

use crate::domain::shared::models::{AccountId, UserId};
use crate::domain::user_info::models::{Avatar, AvatarInfo, PlatformImage};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait AvatarRepository: SendUnlessWasm + SyncUnlessWasm {
    /// Loads the avatar for `user_jid` and `checksum` and caches it locally.
    async fn precache_avatar_image(
        &self,
        account: &AccountId,
        user_id: &UserId,
        metadata: &Avatar,
    ) -> Result<()>;

    /// Returns the avatar for `user_jid` and `metadata` from cache or loads it from the server.
    async fn get(
        &self,
        account: &AccountId,
        user_id: &UserId,
        metadata: &Avatar,
    ) -> Result<Option<PlatformImage>>;

    /// Saves the avatar to the local cache.
    async fn set(
        &self,
        account: &AccountId,
        user_id: &UserId,
        metadata: &AvatarInfo,
        image: &AvatarData,
    ) -> Result<()>;

    async fn clear_cache(&self, account: &AccountId) -> Result<()>;
}
