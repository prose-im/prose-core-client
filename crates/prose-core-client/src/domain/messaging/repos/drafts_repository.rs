// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::dtos::RoomId;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait DraftsRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn get(&self, room_id: &RoomId) -> Result<Option<String>>;
    async fn set(&self, room_id: &RoomId, draft: Option<&str>) -> Result<()>;
    async fn clear_cache(&self) -> Result<()>;
}
