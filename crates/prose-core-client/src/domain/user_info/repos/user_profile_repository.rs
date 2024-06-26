// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::shared::models::AccountId;
use crate::domain::user_info::models::UserProfile;
use crate::dtos::UserId;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait UserProfileRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn get(&self, account: &AccountId, user_id: &UserId) -> Result<Option<UserProfile>>;
    async fn set(&self, account: &AccountId, user_id: &UserId, profile: &UserProfile)
        -> Result<()>;
    async fn delete(&self, account: &AccountId, user_id: &UserId) -> Result<()>;

    /// Returns the display name for `user_id`. Display name is a cascade of first_name, last_name
    /// and nickname;
    async fn get_display_name(
        &self,
        account: &AccountId,
        user_id: &UserId,
    ) -> Result<Option<String>>;

    async fn reset_before_reconnect(&self, account: &AccountId) -> Result<()>;
    async fn clear_cache(&self, account: &AccountId) -> Result<()>;
}
