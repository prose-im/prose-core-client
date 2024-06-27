// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::mods::AvatarData;

use crate::domain::shared::models::{AvatarId, UserId};
use crate::domain::user_info::models::AvatarMetadata;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait UserInfoService: SendUnlessWasm + SyncUnlessWasm {
    async fn load_latest_avatar_metadata(&self, from: &UserId) -> Result<Option<AvatarMetadata>>;
    async fn load_avatar_image(
        &self,
        from: &UserId,
        image_id: &AvatarId,
    ) -> Result<Option<AvatarData>>;
}
