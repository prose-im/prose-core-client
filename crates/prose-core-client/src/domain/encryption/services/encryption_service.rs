// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use std::time::SystemTime;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::encryption::models::{DeviceId, LocalEncryptionBundle, PreKeyBundle};
use crate::domain::messaging::models::send_message_request::EncryptedMessage;
use crate::domain::shared::models::UserId;
use crate::dtos::{PreKeyId, PreKeyRecord};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait EncryptionService: SendUnlessWasm + SyncUnlessWasm {
    async fn generate_local_encryption_bundle(
        &self,
        device_id: DeviceId,
    ) -> Result<LocalEncryptionBundle>;

    async fn generate_pre_keys_with_ids(&self, ids: Vec<PreKeyId>) -> Result<Vec<PreKeyRecord>>;

    async fn process_pre_key_bundle(&self, user_id: &UserId, bundle: PreKeyBundle) -> Result<()>;

    async fn encrypt_key(
        &self,
        recipient_id: &UserId,
        device_id: &DeviceId,
        message: &[u8],
        now: &SystemTime,
    ) -> Result<EncryptedMessage>;

    async fn decrypt_key(
        &self,
        sender_id: &UserId,
        device_id: &DeviceId,
        message: &[u8],
        is_pre_key: bool,
    ) -> Result<Box<[u8]>>;
}
