// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;

use crate::dtos::UserId;
use anyhow::Result;
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait BlockListDomainService: SendUnlessWasm + SyncUnlessWasm {
    async fn load_block_list(&self) -> Result<Vec<UserId>>;
    async fn block_user(&self, user_id: &UserId) -> Result<()>;
    async fn unblock_user(&self, user_id: &UserId) -> Result<()>;
    async fn clear_block_list(&self) -> Result<()>;

    async fn handle_user_blocked(&self, user_id: &UserId) -> Result<()>;
    async fn handle_user_unblocked(&self, user_id: &UserId) -> Result<()>;
    async fn handle_block_list_cleared(&self) -> Result<()>;

    async fn clear_cache(&self) -> Result<()>;
}
