// prose-core-client/prose-core-client
//
// Copyright: 2025, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::CachePolicy;
use crate::domain::user_info::models::PlatformImage;
use crate::domain::workspace::models::{WorkspaceIcon, WorkspaceInfo};
use anyhow::Result;
use async_trait::async_trait;
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait WorkspaceInfoDomainService: SendUnlessWasm + SyncUnlessWasm {
    async fn get_workspace_info(&self, cache_policy: CachePolicy) -> Result<Option<WorkspaceInfo>>;
    async fn load_workspace_icon(&self, icon: &WorkspaceIcon) -> Result<Option<PlatformImage>>;

    async fn handle_workspace_info_changed(&self, info: WorkspaceInfo) -> Result<()>;
    async fn handle_icon_changed(&self, icon: Option<WorkspaceIcon>) -> Result<()>;

    async fn reset_before_reconnect(&self) -> Result<()>;
    async fn clear_cache(&self) -> Result<()>;
}
