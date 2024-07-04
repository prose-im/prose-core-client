// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::shared::models::{AccountId, ParticipantIdRef};
use crate::domain::user_info::models::UserProfile;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait UserProfileRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn get(
        &self,
        account: &AccountId,
        participant_id: ParticipantIdRef<'_>,
    ) -> Result<Option<UserProfile>>;
    async fn set(
        &self,
        account: &AccountId,
        participant_id: ParticipantIdRef<'_>,
        profile: Option<&UserProfile>,
    ) -> Result<()>;

    async fn reset_before_reconnect(&self, account: &AccountId) -> Result<()>;
    async fn clear_cache(&self, account: &AccountId) -> Result<()>;
}
