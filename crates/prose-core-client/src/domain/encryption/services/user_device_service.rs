// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::encryption::models::{DeviceBundle, DeviceId, DeviceList};
use crate::domain::shared::models::UserId;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait UserDeviceService: SendUnlessWasm + SyncUnlessWasm {
    async fn load_device_list(&self, user_id: &UserId) -> Result<DeviceList>;
    async fn publish_device_list(&self, device_list: DeviceList) -> Result<()>;

    async fn load_device_bundle(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
    ) -> Result<Option<DeviceBundle>>;
    async fn publish_device_bundle(&self, bundle: DeviceBundle) -> Result<()>;
}
