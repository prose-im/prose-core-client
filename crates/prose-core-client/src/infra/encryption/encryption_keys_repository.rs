// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn};
use uuid::Uuid;

use prose_store::prelude::*;

use crate::domain::encryption::models::{
    DeviceId, EncryptionDirection, IdentityKey, KyberPreKeyId, KyberPreKeyRecord, LocalDevice,
    LocalEncryptionBundle, PreKeyId, PreKeyRecord, SenderKeyRecord, SessionRecord, SignedPreKeyId,
    SignedPreKeyRecord,
};
use crate::domain::encryption::repos::EncryptionKeysRepository as EncryptionKeysRepositoryTrait;
use crate::dtos::{DeviceBundle, UserId};
use crate::infra::encryption::encryption_key_records::LocalDeviceRecord;
use crate::infra::encryption::user_device_key::SenderDistributionKeyRef;
use crate::infra::encryption::{encryption_keys_collections, UserDeviceKey, UserDeviceKeyRef};

pub struct EncryptionKeysRepository {
    store: Store<PlatformDriver>,
}

impl EncryptionKeysRepository {
    pub fn new(store: Store<PlatformDriver>) -> Self {
        Self { store }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl EncryptionKeysRepositoryTrait for EncryptionKeysRepository {
    async fn put_local_encryption_bundle(&self, bundle: &LocalEncryptionBundle) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[
                encryption_keys_collections::LOCAL_DEVICE,
                encryption_keys_collections::SIGNED_PRE_KEY,
                encryption_keys_collections::PRE_KEY,
            ])
            .await?;

        {
            let collection = tx.writeable_collection(encryption_keys_collections::LOCAL_DEVICE)?;
            let record = LocalDeviceRecord {
                device_id: bundle.device_id.clone(),
                identity_key_pair: bundle.identity_key_pair.clone(),
            };
            collection.put(LocalDeviceRecord::current_id(), &record)?;
        }

        {
            let collection =
                tx.writeable_collection(encryption_keys_collections::SIGNED_PRE_KEY)?;
            collection.put(bundle.signed_pre_key.id.as_ref(), &bundle.signed_pre_key)?;
        }

        {
            let collection = tx.writeable_collection(encryption_keys_collections::PRE_KEY)?;
            for pre_key in &bundle.pre_keys {
                collection.put(pre_key.id.as_ref(), pre_key)?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get_local_device_bundle(&self) -> Result<Option<DeviceBundle>> {
        let tx = self
            .store
            .transaction_for_reading(&[
                encryption_keys_collections::LOCAL_DEVICE,
                encryption_keys_collections::SIGNED_PRE_KEY,
                encryption_keys_collections::PRE_KEY,
            ])
            .await?;

        let local_device = {
            let collection = tx.readable_collection(encryption_keys_collections::LOCAL_DEVICE)?;
            collection
                .get::<_, LocalDeviceRecord>(LocalDeviceRecord::current_id())
                .await?
        };

        let Some(local_device) = local_device else {
            return Ok(None);
        };

        let signed_pre_key = {
            let collection = tx.readable_collection(encryption_keys_collections::SIGNED_PRE_KEY)?;
            collection
                .get_all_values::<SignedPreKeyRecord>(
                    Query::<u32>::All,
                    QueryDirection::Backward,
                    Some(1),
                )
                .await?
        };

        let Some(signed_pre_key) = signed_pre_key.into_iter().next() else {
            return Ok(None);
        };

        let pre_keys = {
            let collection = tx.readable_collection(encryption_keys_collections::PRE_KEY)?;
            collection
                .get_all_values::<PreKeyRecord>(Query::<u32>::All, QueryDirection::Forward, None)
                .await?
        };

        Ok(Some(DeviceBundle {
            device_id: local_device.device_id,
            signed_pre_key: signed_pre_key.into_public_signed_pre_key(),
            identity_key: local_device.identity_key_pair.identity_key,
            pre_keys: pre_keys
                .into_iter()
                .map(|key| key.into_public_pre_key())
                .collect(),
        }))
    }

    async fn get_local_device(&self) -> Result<Option<LocalDevice>> {
        let tx = self
            .store
            .transaction_for_reading(&[encryption_keys_collections::LOCAL_DEVICE])
            .await?;

        let local_device = {
            let collection = tx.readable_collection(encryption_keys_collections::LOCAL_DEVICE)?;
            collection
                .get::<_, LocalDeviceRecord>(LocalDeviceRecord::current_id())
                .await?
        };

        Ok(local_device.map(LocalDevice::from))
    }

    async fn get_session(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
    ) -> Result<Option<SessionRecord>> {
        let tx = self
            .store
            .transaction_for_reading(&[encryption_keys_collections::SESSION_RECORD])
            .await?;
        let collection = tx.readable_collection(encryption_keys_collections::SESSION_RECORD)?;

        let key = UserDeviceKeyRef::new(user_id, device_id);
        let session = collection.get(&key).await?;

        if session.is_none() {
            warn!("Could not find session for {user_id} ({device_id}).")
        }

        Ok(session)
    }

    async fn put_session(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
        record: &SessionRecord,
    ) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[encryption_keys_collections::SESSION_RECORD])
            .await?;
        let collection = tx.writeable_collection(encryption_keys_collections::SESSION_RECORD)?;

        let key = UserDeviceKeyRef::new(user_id, device_id);
        collection.put(&key, record)?;
        tx.commit().await?;
        Ok(())
    }

    async fn get_active_device_ids(&self, user_id: &UserId) -> Result<Vec<DeviceId>> {
        let tx = self
            .store
            .transaction_for_reading(&[encryption_keys_collections::SESSION_RECORD])
            .await?;

        let collection = tx.readable_collection(encryption_keys_collections::SESSION_RECORD)?;
        let session_ids = collection
            .get_all_filtered::<IdentityKey, _>(
                Query::from_range(UserDeviceKey::min(user_id)..=UserDeviceKey::max(user_id)),
                QueryDirection::Forward,
                None,
                |key, _| Some(key),
            )
            .await?;

        let device_ids = session_ids
            .into_iter()
            .map(|key| UserDeviceKey::parse_device_id_from_key(&key))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(device_ids)
    }

    async fn get_kyber_pre_key(
        &self,
        kyber_prekey_id: KyberPreKeyId,
    ) -> Result<Option<KyberPreKeyRecord>> {
        let tx = self
            .store
            .transaction_for_reading(&[encryption_keys_collections::KYBER_PRE_KEY])
            .await?;
        let collection = tx.readable_collection(encryption_keys_collections::KYBER_PRE_KEY)?;

        let kyber_pre_key = collection.get(kyber_prekey_id.as_ref()).await?;
        Ok(kyber_pre_key)
    }

    async fn put_kyber_pre_key(
        &self,
        kyber_prekey_id: KyberPreKeyId,
        record: &KyberPreKeyRecord,
    ) -> Result<()> {
        self.store
            .put(
                encryption_keys_collections::KYBER_PRE_KEY,
                kyber_prekey_id.as_ref(),
                record,
            )
            .await?;
        Ok(())
    }

    async fn delete_kyber_pre_key(&self, kyber_prekey_id: KyberPreKeyId) -> Result<()> {
        self.store
            .delete(
                encryption_keys_collections::KYBER_PRE_KEY,
                kyber_prekey_id.as_ref(),
            )
            .await?;
        Ok(())
    }

    async fn get_signed_pre_key(
        &self,
        signed_prekey_id: SignedPreKeyId,
    ) -> Result<Option<SignedPreKeyRecord>> {
        let tx = self
            .store
            .transaction_for_reading(&[encryption_keys_collections::SIGNED_PRE_KEY])
            .await?;
        let collection = tx.readable_collection(encryption_keys_collections::SIGNED_PRE_KEY)?;
        let key = collection.get(signed_prekey_id.as_ref()).await?;
        Ok(key)
    }

    async fn put_signed_pre_key(&self, record: &SignedPreKeyRecord) -> Result<()> {
        self.store
            .put(
                encryption_keys_collections::SIGNED_PRE_KEY,
                record.id.as_ref(),
                record,
            )
            .await?;
        Ok(())
    }

    async fn delete_signed_pre_key(&self, signed_prekey_id: SignedPreKeyId) -> Result<()> {
        self.store
            .delete(
                encryption_keys_collections::SIGNED_PRE_KEY,
                signed_prekey_id.as_ref(),
            )
            .await?;
        Ok(())
    }

    async fn get_pre_key(&self, prekey_id: PreKeyId) -> Result<Option<PreKeyRecord>> {
        let tx = self
            .store
            .transaction_for_reading(&[encryption_keys_collections::PRE_KEY])
            .await?;
        let collection = tx.readable_collection(encryption_keys_collections::PRE_KEY)?;

        let pre_key = collection.get(prekey_id.as_ref()).await?;
        Ok(pre_key)
    }

    async fn put_pre_keys(&self, records: &[PreKeyRecord]) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[encryption_keys_collections::PRE_KEY])
            .await?;
        let collection = tx.writeable_collection(encryption_keys_collections::PRE_KEY)?;

        for pre_key in records {
            collection.put(pre_key.id.as_ref(), pre_key)?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get_all_pre_keys(&self) -> Result<Vec<PreKeyRecord>> {
        let tx = self
            .store
            .transaction_for_reading(&[encryption_keys_collections::PRE_KEY])
            .await?;
        let collection = tx.readable_collection(encryption_keys_collections::PRE_KEY)?;

        let pre_keys = collection
            .get_all_values::<PreKeyRecord>(Query::<u32>::All, QueryDirection::Forward, None)
            .await?;
        Ok(pre_keys)
    }

    async fn delete_pre_key(&self, prekey_id: PreKeyId) -> Result<()> {
        info!("Deleting PreKey with id {:?}…", prekey_id);
        self.store
            .delete(encryption_keys_collections::PRE_KEY, prekey_id.as_ref())
            .await?;
        Ok(())
    }

    async fn put_sender_key(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
        distribution_id: Uuid,
        record: &SenderKeyRecord,
    ) -> Result<()> {
        let key = SenderDistributionKeyRef::new(user_id, device_id, &distribution_id);
        self.store
            .put(encryption_keys_collections::SENDER_KEY, &key, record)
            .await?;
        Ok(())
    }

    async fn get_sender_key(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
        distribution_id: Uuid,
    ) -> Result<Option<SenderKeyRecord>> {
        let tx = self
            .store
            .transaction_for_reading(&[encryption_keys_collections::SENDER_KEY])
            .await?;

        let collection = tx.readable_collection(encryption_keys_collections::SENDER_KEY)?;
        let key = SenderDistributionKeyRef::new(user_id, device_id, &distribution_id);
        let record = collection.get(&key).await?;
        Ok(record)
    }

    async fn save_identity(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
        identity: &IdentityKey,
    ) -> Result<bool> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[encryption_keys_collections::IDENTITY])
            .await?;
        let collection = tx.writeable_collection(encryption_keys_collections::IDENTITY)?;

        let key = UserDeviceKeyRef::new(user_id, device_id);

        if Some(identity) == collection.get(&key).await?.as_ref() {
            return Ok(true);
        }

        collection.put(&key, identity)?;
        tx.commit().await?;

        Ok(false)
    }

    /// Return whether an identity is trusted for the role specified by `direction`.
    async fn is_trusted_identity(
        &self,
        user_id: &UserId,
        device_id: Option<&DeviceId>,
        identity: &IdentityKey,
        _direction: EncryptionDirection,
    ) -> Result<bool> {
        if let Some(device_id) = device_id {
            return Ok(self.get_identity(user_id, device_id).await?.as_ref() == Some(identity));
        };

        let tx = self
            .store
            .transaction_for_reading(&[encryption_keys_collections::IDENTITY])
            .await?;
        let collection = tx.readable_collection(encryption_keys_collections::IDENTITY)?;

        let matching_identities = collection
            .get_all_filtered::<IdentityKey, _>(
                Query::from_range(UserDeviceKey::min(user_id)..=UserDeviceKey::max(user_id)),
                QueryDirection::Forward,
                None,
                |_, value| (&value == identity).then_some(true),
            )
            .await?;

        Ok(!matching_identities.is_empty())
    }

    /// Return the public identity for the given `address`, if known.
    async fn get_identity(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
    ) -> Result<Option<IdentityKey>> {
        let tx = self
            .store
            .transaction_for_reading(&[encryption_keys_collections::IDENTITY])
            .await?;
        let collection = tx.readable_collection(encryption_keys_collections::IDENTITY)?;

        let key = UserDeviceKeyRef::new(user_id, device_id);
        let identity = collection.get(&key).await?;
        Ok(identity)
    }
}
