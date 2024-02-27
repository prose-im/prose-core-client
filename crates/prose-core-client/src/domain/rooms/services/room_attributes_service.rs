// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::shared::models::MucId;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait RoomAttributesService: SendUnlessWasm + SyncUnlessWasm {
    async fn set_topic(&self, room_id: &MucId, subject: Option<&str>) -> Result<()>;
    async fn set_name(&self, room_id: &MucId, name: &str) -> Result<()>;
}
