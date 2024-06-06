// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::encryption::models::{
    DeviceId, KyberPreKey, KyberPreKeyId, LocalDevice, LocalEncryptionBundle, PreKey, PreKeyId,
    SenderKey, SignedPreKey, SignedPreKeyId,
};
use crate::domain::shared::models::AccountId;
use crate::dtos::{DeviceBundle, UserId};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait EncryptionKeysRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn put_local_encryption_bundle(
        &self,
        account: &AccountId,
        bundle: &LocalEncryptionBundle,
    ) -> Result<()>;

    async fn get_local_device_bundle(&self, account: &AccountId) -> Result<Option<DeviceBundle>>;
    async fn get_local_device(&self, account: &AccountId) -> Result<Option<LocalDevice>>;

    async fn get_kyber_pre_key(
        &self,
        account: &AccountId,
        kyber_prekey_id: KyberPreKeyId,
    ) -> Result<Option<KyberPreKey>>;
    async fn put_kyber_pre_key(
        &self,
        account: &AccountId,
        kyber_prekey_id: KyberPreKeyId,
        record: &KyberPreKey,
    ) -> Result<()>;
    async fn delete_kyber_pre_key(
        &self,
        account: &AccountId,
        kyber_prekey_id: KyberPreKeyId,
    ) -> Result<()>;

    async fn get_signed_pre_key(
        &self,
        account: &AccountId,
        signed_prekey_id: SignedPreKeyId,
    ) -> Result<Option<SignedPreKey>>;
    async fn put_signed_pre_key(&self, account: &AccountId, record: &SignedPreKey) -> Result<()>;
    async fn delete_signed_pre_key(
        &self,
        account: &AccountId,
        signed_prekey_id: SignedPreKeyId,
    ) -> Result<()>;

    async fn get_pre_key(&self, account: &AccountId, prekey_id: PreKeyId)
        -> Result<Option<PreKey>>;
    async fn put_pre_keys(&self, account: &AccountId, records: &[PreKey]) -> Result<()>;

    async fn put_sender_key(
        &self,
        account: &AccountId,
        user_id: &UserId,
        device_id: &DeviceId,
        distribution_id: Uuid,
        record: &SenderKey,
    ) -> Result<()>;
    async fn get_sender_key(
        &self,
        account: &AccountId,
        user_id: &UserId,
        device_id: &DeviceId,
        distribution_id: Uuid,
    ) -> Result<Option<SenderKey>>;

    async fn clear_cache(&self, account: &AccountId) -> Result<()>;
}
