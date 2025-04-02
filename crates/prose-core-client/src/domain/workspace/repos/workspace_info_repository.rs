// prose-core-client/prose-core-client
//
// Copyright: 2025, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::shared::models::AccountId;
use crate::domain::workspace::models::WorkspaceInfo;

pub type UpdateHandler = Box<dyn FnOnce(&mut WorkspaceInfo) + Send>;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait WorkspaceInfoRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn get(&self, account: &AccountId) -> Result<Option<WorkspaceInfo>>;

    // Upserts `WorkspaceInfo`. Returns `true` if the `WorkspaceInfo` was changed
    // after executing `handler`.
    async fn update(&self, account: &AccountId, handler: UpdateHandler) -> Result<bool>;

    async fn clear_cache(&self, account: &AccountId) -> Result<()>;
}
