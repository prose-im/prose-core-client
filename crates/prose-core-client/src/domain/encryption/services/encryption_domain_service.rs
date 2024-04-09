// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::encryption::models::{DeviceInfo, DeviceList};
use crate::domain::messaging::models::{send_message_request::EncryptedPayload, MessageId};
use crate::domain::shared::models::UserId;
use crate::dtos::DeviceId;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait EncryptionDomainService: SendUnlessWasm + SyncUnlessWasm {
    async fn initialize(&self) -> Result<()>;

    async fn start_session(&self, user_id: &UserId) -> Result<()>;

    async fn encrypt_message(
        &self,
        recipient_id: &UserId,
        message: String,
    ) -> Result<EncryptedPayload>;
    async fn decrypt_message(
        &self,
        sender_id: &UserId,
        message_id: &MessageId,
        payload: EncryptedPayload,
    ) -> Result<String>;

    async fn load_device_infos(&self, user_id: &UserId) -> Result<Vec<DeviceInfo>>;
    async fn delete_device(&self, device_id: &DeviceId) -> Result<()>;
    async fn disable_omemo(&self) -> Result<()>;

    async fn handle_received_device_list(
        &self,
        user_id: &UserId,
        device_list: DeviceList,
    ) -> Result<()>;

    async fn clear_cache(&self) -> Result<()>;
}
