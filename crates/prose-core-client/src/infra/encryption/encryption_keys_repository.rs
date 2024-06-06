// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use prose_store::prelude::*;

use crate::domain::encryption::models::{
    DeviceId, KyberPreKey, KyberPreKeyId, LocalDevice, LocalEncryptionBundle, PreKey, PreKeyId,
    SenderKey, SignedPreKey, SignedPreKeyId,
};
use crate::domain::encryption::repos::EncryptionKeysRepository as EncryptionKeysRepositoryTrait;
use crate::domain::shared::models::AccountId;
use crate::dtos::{DeviceBundle, UserId};
use crate::infra::encryption::{
    KyberPreKeyRecord, LocalDeviceRecord, PreKeyRecord, SenderKeyRecord, SignedPreKeyRecord,
};

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
    async fn put_local_encryption_bundle(
        &self,
        account: &AccountId,
        bundle: &LocalEncryptionBundle,
    ) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[
                LocalDeviceRecord::collection(),
                SignedPreKeyRecord::collection(),
                PreKeyRecord::collection(),
            ])
            .await?;

        {
            let collection = tx.writeable_collection(LocalDeviceRecord::collection())?;
            let record = LocalDeviceRecord::new(
                account,
                &bundle.device_id,
                bundle.identity_key_pair.clone(),
            );
            collection.put_entity(&record)?;
        }

        {
            let collection = tx.writeable_collection(SignedPreKeyRecord::collection())?;
            let record = SignedPreKeyRecord::new(account, bundle.signed_pre_key.clone());
            collection.put_entity(&record)?;
        }

        {
            let collection = tx.writeable_collection(PreKeyRecord::collection())?;
            for pre_key in &bundle.pre_keys {
                let record = PreKeyRecord::new(account, pre_key.clone());
                collection.put_entity(&record)?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get_local_device_bundle(&self, account: &AccountId) -> Result<Option<DeviceBundle>> {
        let tx = self
            .store
            .transaction_for_reading(&[
                LocalDeviceRecord::collection(),
                SignedPreKeyRecord::collection(),
                PreKeyRecord::collection(),
            ])
            .await?;

        let local_device = {
            let collection = tx.readable_collection(LocalDeviceRecord::collection())?;
            collection.get::<_, LocalDeviceRecord>(account).await?
        }
        .map(LocalDevice::from);

        let Some(local_device) = local_device else {
            return Ok(None);
        };

        let signed_pre_key = {
            let collection = tx.readable_collection(SignedPreKeyRecord::collection())?;
            let idx = collection.index(&SignedPreKeyRecord::account_idx())?;
            idx.get_all_values::<SignedPreKeyRecord>(
                Query::Only(account),
                QueryDirection::Backward,
                Some(1),
            )
            .await?
        };

        let Some(signed_pre_key) = signed_pre_key.into_iter().next() else {
            return Ok(None);
        };

        let pre_keys = {
            let collection = tx.readable_collection(PreKeyRecord::collection())?;
            let idx = collection.index(&PreKeyRecord::account_idx())?;
            idx.get_all_values::<PreKeyRecord>(Query::Only(account), QueryDirection::Forward, None)
                .await?
        };

        Ok(Some(DeviceBundle {
            device_id: local_device.device_id,
            signed_pre_key: SignedPreKey::from(signed_pre_key).into_public_signed_pre_key(),
            identity_key: local_device.identity_key_pair.identity_key,
            pre_keys: pre_keys
                .into_iter()
                .map(|key| PreKey::from(key).into_public_pre_key())
                .collect(),
        }))
    }

    async fn get_local_device(&self, account: &AccountId) -> Result<Option<LocalDevice>> {
        let tx = self
            .store
            .transaction_for_reading(&[LocalDeviceRecord::collection()])
            .await?;

        let local_device = {
            let collection = tx.readable_collection(LocalDeviceRecord::collection())?;
            collection.get::<_, LocalDeviceRecord>(account).await?
        };

        Ok(local_device.map(LocalDevice::from))
    }

    async fn get_kyber_pre_key(
        &self,
        account: &AccountId,
        kyber_prekey_id: KyberPreKeyId,
    ) -> Result<Option<KyberPreKey>> {
        let tx = self
            .store
            .transaction_for_reading(&[KyberPreKeyRecord::collection()])
            .await?;
        let collection = tx.readable_collection(KyberPreKeyRecord::collection())?;
        let idx = collection.index(&KyberPreKeyRecord::pre_key_idx())?;
        let kyber_pre_key = idx
            .get::<_, KyberPreKeyRecord>(&(account, kyber_prekey_id.as_ref()))
            .await?;
        Ok(kyber_pre_key.map(Into::into))
    }

    async fn put_kyber_pre_key(
        &self,
        account: &AccountId,
        kyber_prekey_id: KyberPreKeyId,
        record: &KyberPreKey,
    ) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[KyberPreKeyRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(KyberPreKeyRecord::collection())?;
        collection.put_entity(&KyberPreKeyRecord::new(
            account,
            &kyber_prekey_id,
            record.clone(),
        ))?;
        tx.commit().await?;
        Ok(())
    }

    async fn delete_kyber_pre_key(
        &self,
        account: &AccountId,
        kyber_prekey_id: KyberPreKeyId,
    ) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[KyberPreKeyRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(KyberPreKeyRecord::collection())?;
        let idx = collection.index(&KyberPreKeyRecord::pre_key_idx())?;
        idx.delete(&(account, kyber_prekey_id.as_ref())).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn get_signed_pre_key(
        &self,
        account: &AccountId,
        signed_prekey_id: SignedPreKeyId,
    ) -> Result<Option<SignedPreKey>> {
        let tx = self
            .store
            .transaction_for_reading(&[SignedPreKeyRecord::collection()])
            .await?;
        let collection = tx.readable_collection(SignedPreKeyRecord::collection())?;
        let idx = collection.index(&SignedPreKeyRecord::pre_key_idx())?;
        let key = idx
            .get::<_, SignedPreKeyRecord>(&(account, signed_prekey_id.as_ref()))
            .await?;
        Ok(key.map(Into::into))
    }

    async fn put_signed_pre_key(&self, account: &AccountId, record: &SignedPreKey) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[SignedPreKeyRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(SignedPreKeyRecord::collection())?;
        collection.put_entity(&SignedPreKeyRecord::new(account, record.clone()))?;
        tx.commit().await?;
        Ok(())
    }

    async fn delete_signed_pre_key(
        &self,
        account: &AccountId,
        signed_prekey_id: SignedPreKeyId,
    ) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[SignedPreKeyRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(SignedPreKeyRecord::collection())?;
        let idx = collection.index(&SignedPreKeyRecord::pre_key_idx())?;
        idx.delete(&(account, signed_prekey_id.as_ref())).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn get_pre_key(
        &self,
        account: &AccountId,
        prekey_id: PreKeyId,
    ) -> Result<Option<PreKey>> {
        let tx = self
            .store
            .transaction_for_reading(&[PreKeyRecord::collection()])
            .await?;
        let collection = tx.readable_collection(PreKeyRecord::collection())?;
        let idx = collection.index(&PreKeyRecord::pre_key_idx())?;
        let pre_key = idx
            .get::<_, PreKeyRecord>(&(account, prekey_id.as_ref()))
            .await?;
        Ok(pre_key.map(Into::into))
    }

    async fn put_pre_keys(&self, account: &AccountId, pre_keys: &[PreKey]) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[PreKeyRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(PreKeyRecord::collection())?;

        for pre_key in pre_keys {
            collection.put_entity(&PreKeyRecord::new(account, pre_key.clone()))?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn put_sender_key(
        &self,
        account: &AccountId,
        user_id: &UserId,
        device_id: &DeviceId,
        distribution_id: Uuid,
        record: &SenderKey,
    ) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[SenderKeyRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(SenderKeyRecord::collection())?;
        collection.put_entity(&SenderKeyRecord::new(
            account,
            user_id,
            device_id,
            distribution_id,
            record.clone(),
        ))?;
        tx.commit().await?;
        Ok(())
    }

    async fn get_sender_key(
        &self,
        account: &AccountId,
        user_id: &UserId,
        device_id: &DeviceId,
        distribution_id: Uuid,
    ) -> Result<Option<SenderKey>> {
        let tx = self
            .store
            .transaction_for_reading(&[SenderKeyRecord::collection()])
            .await?;
        let collection = tx.readable_collection(SenderKeyRecord::collection())?;
        let idx = collection.index(&SenderKeyRecord::distribution_idx())?;
        let record = idx
            .get::<_, SenderKeyRecord>(&(account, user_id, device_id, distribution_id))
            .await?;
        Ok(record.map(Into::into))
    }

    async fn clear_cache(&self, account: &AccountId) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[
                LocalDeviceRecord::collection(),
                KyberPreKeyRecord::collection(),
                PreKeyRecord::collection(),
                SenderKeyRecord::collection(),
                SignedPreKeyRecord::collection(),
            ])
            .await?;

        let collection = tx.writeable_collection(LocalDeviceRecord::collection())?;
        collection.delete(account).await?;

        let collection = tx.writeable_collection(KyberPreKeyRecord::collection())?;
        collection
            .delete_all_in_index(&KyberPreKeyRecord::account_idx(), Query::Only(account))
            .await?;

        let collection = tx.writeable_collection(PreKeyRecord::collection())?;
        collection
            .delete_all_in_index(&PreKeyRecord::account_idx(), Query::Only(account))
            .await?;

        let collection = tx.writeable_collection(SenderKeyRecord::collection())?;
        collection
            .delete_all_in_index(&SenderKeyRecord::account_idx(), Query::Only(account))
            .await?;

        let collection = tx.writeable_collection(SignedPreKeyRecord::collection())?;
        collection
            .delete_all_in_index(&SignedPreKeyRecord::account_idx(), Query::Only(account))
            .await?;

        tx.commit().await?;
        Ok(())
    }
}
