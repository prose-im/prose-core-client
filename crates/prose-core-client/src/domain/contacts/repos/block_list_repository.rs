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
pub trait BlockListRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn contains(&self, account: &AccountId, user_id: &UserId) -> Result<bool>;
    async fn get_all(&self, account: &AccountId) -> Result<Vec<UserId>>;
    async fn insert(&self, account: &AccountId, user_id: &UserId) -> Result<bool>;
    async fn delete(&self, account: &AccountId, user_id: &UserId) -> Result<bool>;
    async fn delete_all(&self, account: &AccountId) -> Result<bool>;

    async fn reset_before_reconnect(&self, account: &AccountId) -> Result<()>;
    async fn clear_cache(&self, account: &AccountId) -> Result<()>;
}
