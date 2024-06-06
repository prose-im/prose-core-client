// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::shared::models::{AccountId, UserId};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait PresenceSubRequestsRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn get_all(&self, account: &AccountId) -> Result<Vec<UserId>>;
    /// Returns whether the value was newly inserted.
    async fn set(&self, account: &AccountId, user_id: &UserId) -> Result<bool>;
    /// Returns whether the value was present.
    async fn delete(&self, account: &AccountId, user_id: &UserId) -> Result<bool>;

    async fn clear_cache(&self, account: &AccountId) -> Result<()>;
}
