// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::encryption::models::{
    DeviceId, IdentityKey, KyberPreKeyId, KyberPreKeyRecord, LocalDevice, LocalEncryptionBundle,
    PreKeyId, PreKeyRecord, SenderKeyRecord, Session, SessionData, SignedPreKeyId,
    SignedPreKeyRecord,
};
use crate::dtos::{DeviceBundle, UserId};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait EncryptionKeysRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn put_local_encryption_bundle(&self, bundle: &LocalEncryptionBundle) -> Result<()>;

    async fn get_local_device_bundle(&self) -> Result<Option<DeviceBundle>>;
    async fn get_local_device(&self) -> Result<Option<LocalDevice>>;

    async fn get_session(&self, user_id: &UserId, device_id: &DeviceId) -> Result<Option<Session>>;
    async fn get_all_sessions(&self, user_id: &UserId) -> Result<Vec<Session>>;

    async fn put_session_data(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
        data: SessionData,
    ) -> Result<()>;

    /// Record an identity into the store. The identity is then considered "trusted".
    ///
    /// The return value represents whether an existing identity was replaced (`Ok(true)`). If it is
    /// new or hasn't changed, the return value should be `Ok(false)`.
    async fn put_identity(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
        identity: IdentityKey,
    ) -> Result<bool>;

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
