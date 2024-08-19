// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::mods::AvatarData;

use crate::domain::shared::models::{AccountId, AvatarId, ParticipantIdRef};
use crate::domain::user_info::models::{AvatarInfo, PlatformImage};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait AvatarRepository: SendUnlessWasm + SyncUnlessWasm {
    /// Returns the cached avatar.
    async fn get(
        &self,
        account: &AccountId,
        participant_id: ParticipantIdRef<'_>,
        avatar_id: &AvatarId,
    ) -> Result<Option<PlatformImage>>;

    /// Saves the avatar to the local cache.
    async fn set(
        &self,
        account: &AccountId,
        participant_id: ParticipantIdRef<'_>,
        metadata: &AvatarInfo,
        image: &AvatarData,
    ) -> Result<()>;

    async fn clear_cache(&self, account: &AccountId) -> Result<()>;
}
