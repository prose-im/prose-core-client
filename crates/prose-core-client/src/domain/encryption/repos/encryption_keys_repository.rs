// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::encryption::models::{
    DeviceId, KyberPreKeyId, KyberPreKeyRecord, LocalDevice, LocalEncryptionBundle, PreKeyId,
    PreKeyRecord, SenderKeyRecord, SignedPreKeyId, SignedPreKeyRecord,
};
use crate::dtos::{DeviceBundle, UserId};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait EncryptionKeysRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn put_local_encryption_bundle(&self, bundle: &LocalEncryptionBundle) -> Result<()>;

    async fn get_local_device_bundle(&self) -> Result<Option<DeviceBundle>>;
    async fn get_local_device(&self) -> Result<Option<LocalDevice>>;

    async fn get_kyber_pre_key(
        &self,
        kyber_prekey_id: KyberPreKeyId,
    ) -> Result<Option<KyberPreKeyRecord>>;
    async fn put_kyber_pre_key(
        &self,
        kyber_prekey_id: KyberPreKeyId,
        record: &KyberPreKeyRecord,
    ) -> Result<()>;
    async fn delete_kyber_pre_key(&self, kyber_prekey_id: KyberPreKeyId) -> Result<()>;

    async fn get_signed_pre_key(
        &self,
        signed_prekey_id: SignedPreKeyId,
    ) -> Result<Option<SignedPreKeyRecord>>;
    async fn put_signed_pre_key(&self, record: &SignedPreKeyRecord) -> Result<()>;
    async fn delete_signed_pre_key(&self, signed_prekey_id: SignedPreKeyId) -> Result<()>;

    async fn get_pre_key(&self, prekey_id: PreKeyId) -> Result<Option<PreKeyRecord>>;
    async fn put_pre_keys(&self, records: &[PreKeyRecord]) -> Result<()>;
    async fn get_all_pre_keys(&self) -> Result<Vec<PreKeyRecord>>;
    async fn delete_pre_key(&self, prekey_id: PreKeyId) -> Result<()>;

    async fn put_sender_key(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
        distribution_id: Uuid,
        record: &SenderKeyRecord,
    ) -> Result<()>;
    async fn get_sender_key(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
        distribution_id: Uuid,
    ) -> Result<Option<SenderKeyRecord>>;

    async fn clear_cache(&self) -> Result<()>;
}
