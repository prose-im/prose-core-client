// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::settings::models::SyncedRoomSettings;
use crate::domain::shared::models::RoomId;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait SyncedRoomSettingsService: SendUnlessWasm + SyncUnlessWasm {
    async fn load_settings(&self, room_id: &RoomId) -> Result<Option<SyncedRoomSettings>>;
    async fn save_settings(&self, room_id: &RoomId, settings: &SyncedRoomSettings) -> Result<()>;
    async fn delete_settings(&self, room_id: &RoomId) -> Result<()>;
}
