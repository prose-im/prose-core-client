// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::settings::models::LocalRoomSettings;
use crate::domain::shared::models::{RoomId, UserId};

type UpdateHandler = Box<dyn for<'a> FnOnce(&'a mut LocalRoomSettings) + Send>;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait LocalRoomSettingsRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn get(&self, account: &UserId, room_id: &RoomId) -> Result<LocalRoomSettings>;
    async fn update(&self, account: &UserId, room_id: &RoomId, block: UpdateHandler) -> Result<()>;
    async fn clear_cache(&self, account: &UserId) -> Result<()>;
}
