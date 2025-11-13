use crate::dtos::{
    AccountId, DecryptionContext, DeviceId, EncryptionKey, IdentityKeyPair, LocalEncryptionBundle,
    PreKey, PreKeyBundle, PreKeyId, SignedPreKey, UserId,
};
use crate::EncryptionService;
use anyhow::Result;
use async_trait::async_trait;
use std::time::SystemTime;

pub struct NoopEncryptionService {}

impl NoopEncryptionService {
    pub fn new() -> Self {
        NoopEncryptionService {}
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl EncryptionService for NoopEncryptionService {
    async fn generate_local_encryption_bundle(
        &self,
        _account: &AccountId,
        device_id: DeviceId,
    ) -> Result<LocalEncryptionBundle> {
        Ok(LocalEncryptionBundle {
            device_id,
            identity_key_pair: IdentityKeyPair {
                identity_key: (&[] as &[u8]).into(),
                private_key: (&[] as &[u8]).into(),
            },
            signed_pre_key: SignedPreKey {
                id: 1.into(),
                public_key: (&[] as &[u8]).into(),
                private_key: (&[] as &[u8]).into(),
                signature: Box::new([]),
                timestamp: 0,
            },
            pre_keys: vec![],
        })
    }

    async fn generate_pre_keys_with_ids(
        &self,
        _account: &AccountId,
        _ids: Vec<PreKeyId>,
    ) -> Result<Vec<PreKey>> {
        Ok(vec![])
    }

    async fn process_pre_key_bundle(
        &self,
        _account: &AccountId,
        _user_id: &UserId,
        _bundle: PreKeyBundle,
    ) -> Result<()> {
        Ok(())
    }

    async fn encrypt_key(
        &self,
        _account: &AccountId,
        _recipient_id: &UserId,
        device_id: &DeviceId,
        _message: &[u8],
        _now: &SystemTime,
    ) -> Result<EncryptionKey> {
        Ok(EncryptionKey {
            device_id: device_id.clone(),
            is_pre_key: false,
            data: Box::new([]),
        })
    }

    async fn decrypt_key(
        &self,
        _account: &AccountId,
        _sender_id: &UserId,
        _device_id: &DeviceId,
        _message: &[u8],
        _is_pre_key: bool,
        _decryption_context: DecryptionContext,
    ) -> Result<Box<[u8]>> {
        Ok(Box::new([]))
    }
}
