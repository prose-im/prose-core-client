// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use tracing::debug;

use prose_xmpp::mods;

use crate::domain::encryption::models::{DeviceBundle, DeviceId, DeviceList};
use crate::domain::encryption::services::UserDeviceService;
use crate::dtos::UserId;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl UserDeviceService for XMPPClient {
    async fn load_device_list(&self, user_id: &UserId) -> Result<DeviceList> {
        let omemo = self.client.get_mod::<mods::OMEMO>();
        let device_list = omemo.load_device_list(user_id.as_ref()).await?;
        Ok(device_list.into())
    }

    async fn publish_device_list(&self, device_list: DeviceList) -> Result<()> {
        debug!("Publishing device list…");
        let omemo = self.client.get_mod::<mods::OMEMO>();
        omemo.publish_device_list(device_list.into()).await?;
        Ok(())
    }

    async fn delete_device_list(&self) -> Result<()> {
        let omemo = self.client.get_mod::<mods::OMEMO>();
        omemo.delete_device_list().await?;
        Ok(())
    }

    async fn load_device_bundle(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
    ) -> Result<Option<DeviceBundle>> {
        let omemo = self.client.get_mod::<mods::OMEMO>();
        let bundle = omemo
            .load_device_bundle(user_id.as_ref(), *device_id.as_ref())
            .await?
            .map(|bundle| DeviceBundle::try_from((device_id.clone(), bundle)))
            .transpose()?;
        Ok(bundle)
    }

    async fn publish_device_bundle(&self, bundle: DeviceBundle) -> Result<()> {
        debug!("Publishing device bundle…");
        let omemo = self.client.get_mod::<mods::OMEMO>();
        omemo
            .publish_device_bundle(*bundle.device_id.as_ref(), bundle.into())
            .await?;
        Ok(())
    }

    async fn delete_device_bundle(&self, device_id: &DeviceId) -> Result<()> {
        let omemo = self.client.get_mod::<mods::OMEMO>();
        omemo.delete_device_bundle(*device_id.as_ref()).await?;
        Ok(())
    }
}
