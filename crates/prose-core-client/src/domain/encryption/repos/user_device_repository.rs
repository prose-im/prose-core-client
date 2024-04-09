// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::encryption::models::Device;
use crate::dtos::UserId;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait UserDeviceRepository: SendUnlessWasm + SyncUnlessWasm {
    /// Returns all devices associated with `user_id`.
    async fn get_all(&self, user_id: &UserId) -> Result<Vec<Device>>;
    /// Sets `devices` for `user_id`. Devices not contained in `devices` will be deleted.
    async fn set_all(&self, user_id: &UserId, devices: Vec<Device>) -> Result<()>;
    /// Deletes all cached devices for all users.
    async fn clear_cache(&self) -> Result<()>;
}
