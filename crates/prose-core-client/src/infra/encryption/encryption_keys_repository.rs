// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use tracing::info;
use uuid::Uuid;

use prose_store::prelude::*;

use crate::domain::encryption::models::{
    DeviceId, IdentityKey, KyberPreKeyId, KyberPreKeyRecord, LocalDevice, LocalEncryptionBundle,
    PreKeyId, PreKeyRecord, SenderKeyRecord, Session, SessionData, SignedPreKeyId,
    SignedPreKeyRecord, Trust,
};
use crate::domain::encryption::repos::EncryptionKeysRepository as EncryptionKeysRepositoryTrait;
use crate::dtos::{DeviceBundle, UserId};
use crate::infra::encryption::encryption_key_records::{LocalDeviceRecord, SessionRecord};
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

    async fn get_session(&self, user_id: &UserId, device_id: &DeviceId) -> Result<Option<Session>> {
        let tx = self
            .store
            .transaction_for_reading(&[encryption_keys_collections::SESSION_RECORD])
            .await?;
        let collection = tx.readable_collection(encryption_keys_collections::SESSION_RECORD)?;

        let key = UserDeviceKeyRef::new(user_id, device_id);
        let session = collection.get::<_, SessionRecord>(&key).await?;

        Ok(session.map(Session::from))
    }

    async fn get_all_sessions(&self, user_id: &UserId) -> Result<Vec<Session>> {
        let tx = self
            .store
            .transaction_for_reading(&[encryption_keys_collections::SESSION_RECORD])
            .await?;
        let collection = tx.readable_collection(encryption_keys_collections::SESSION_RECORD)?;

        let records = collection
            .get_all_filtered::<SessionRecord, _>(
                Query::<UserDeviceKey>::All,
                QueryDirection::Forward,
                None,
                |_, record| {
                    if &record.user_id != user_id {
                        return None;
                    }
                    return Some(Session::from(record));
                },
            )
            .await?;

        Ok(records)
    }

    async fn put_session_data(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
        data: SessionData,
    ) -> Result<()> {
        self.upsert_session(user_id, device_id, move |session| session.data = Some(data))
            .await?;
        Ok(())
    }

    async fn put_identity(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
        identity: IdentityKey,
    ) -> Result<bool> {
        self.upsert_session(user_id, device_id, move |session| {
            session.identity = Some(identity)
        })
        .await
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

    async fn clear_cache(&self) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[
                encryption_keys_collections::KYBER_PRE_KEY,
                encryption_keys_collections::LOCAL_DEVICE,
                encryption_keys_collections::PRE_KEY,
                encryption_keys_collections::SENDER_KEY,
                encryption_keys_collections::SESSION_RECORD,
                encryption_keys_collections::SIGNED_PRE_KEY,
            ])
            .await?;
        tx.truncate_collections(&[
            encryption_keys_collections::KYBER_PRE_KEY,
            encryption_keys_collections::LOCAL_DEVICE,
            encryption_keys_collections::PRE_KEY,
            encryption_keys_collections::SENDER_KEY,
            encryption_keys_collections::SESSION_RECORD,
            encryption_keys_collections::SIGNED_PRE_KEY,
        ])?;
        tx.commit().await?;
        Ok(())
    }
}

impl EncryptionKeysRepository {
    /// The return value represents whether an existing identity was replaced (Ok(true)). If it is
    /// new or hasn't changed, the return value should be Ok(false).
    async fn upsert_session<F: FnOnce(&mut SessionRecord)>(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
        handler: F,
    ) -> Result<bool> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[encryption_keys_collections::SESSION_RECORD])
            .await?;
        let collection = tx.writeable_collection(encryption_keys_collections::SESSION_RECORD)?;

        let key = UserDeviceKeyRef::new(user_id, device_id);

        let existing_session_changed = match collection.get::<_, SessionRecord>(&key).await? {
            Some(session) => {
                let mut updated_session = session.clone();
                handler(&mut updated_session);

                if updated_session != session {
                    collection.put(&key, &updated_session)?;
                    tx.commit().await?;
                    true
                } else {
                    false
                }
            }
            None => {
                let mut session = SessionRecord {
                    user_id: user_id.clone(),
                    device_id: device_id.clone(),
                    trust: Trust::Undecided,
                    is_active: true,
                    data: None,
                    identity: None,
                };
                handler(&mut session);
                collection.put(&key, &session)?;
                tx.commit().await?;
                false
            }
        };

        Ok(existing_session_changed)
    }
}
