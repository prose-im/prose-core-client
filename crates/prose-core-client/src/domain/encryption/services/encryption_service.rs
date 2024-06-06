// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::SystemTime;

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::encryption::models::{
    DecryptionContext, DeviceId, LocalEncryptionBundle, PreKeyBundle,
};
use crate::domain::messaging::models::EncryptionKey;
use crate::domain::shared::models::{AccountId, UserId};
use crate::dtos::{PreKey, PreKeyId};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait EncryptionService: SendUnlessWasm + SyncUnlessWasm {
    async fn generate_local_encryption_bundle(
        &self,
        account: &AccountId,
        device_id: DeviceId,
    ) -> Result<LocalEncryptionBundle>;

    async fn generate_pre_keys_with_ids(
        &self,
        account: &AccountId,
        ids: Vec<PreKeyId>,
    ) -> Result<Vec<PreKey>>;

    async fn process_pre_key_bundle(
        &self,
        account: &AccountId,
        user_id: &UserId,
        bundle: PreKeyBundle,
    ) -> Result<()>;

    async fn encrypt_key(
        &self,
        account: &AccountId,
        recipient_id: &UserId,
        device_id: &DeviceId,
        message: &[u8],
        now: &SystemTime,
    ) -> Result<EncryptionKey>;

    async fn decrypt_key(
        &self,
        account: &AccountId,
        sender_id: &UserId,
        device_id: &DeviceId,
        message: &[u8],
        is_pre_key: bool,
        decryption_context: DecryptionContext,
    ) -> Result<Box<[u8]>>;
}
