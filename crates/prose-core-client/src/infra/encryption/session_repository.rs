// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashSet;

use anyhow::Result;
use async_trait::async_trait;

use prose_store::prelude::*;

use crate::domain::encryption::models::{Session, Trust};
use crate::domain::encryption::repos::SessionRepository as SessionRepositoryTrait;
use crate::dtos::{DeviceId, IdentityKey, SessionData, UserId};
use crate::infra::encryption::encryption_key_records::SessionRecord;
use crate::infra::encryption::{encryption_keys_collections, UserDeviceKey, UserDeviceKeyRef};

pub struct SessionRepository {
    store: Store<PlatformDriver>,
}

impl SessionRepository {
    pub fn new(store: Store<PlatformDriver>) -> Self {
        Self { store }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl SessionRepositoryTrait for SessionRepository {
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

    async fn put_active_devices(&self, user_id: &UserId, device_ids: &[DeviceId]) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[encryption_keys_collections::SESSION_RECORD])
            .await?;
        let collection = tx.writeable_collection(encryption_keys_collections::SESSION_RECORD)?;

        let mut records = collection
            .get_all_filtered::<SessionRecord, _>(
                Query::<UserDeviceKey>::All,
                QueryDirection::Forward,
                None,
                |_, record| (&record.user_id == user_id).then_some(record),
            )
            .await?;

        let device_ids = device_ids.into_iter().collect::<HashSet<_>>();

        for record in records.iter_mut() {
            if record.is_active == device_ids.contains(&record.device_id) {
                continue;
            }

            record.is_active = !record.is_active;
            collection.put(&UserDeviceKeyRef::new(user_id, &record.device_id), &record)?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn clear_cache(&self) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[encryption_keys_collections::SESSION_RECORD])
            .await?;
        tx.truncate_collections(&[encryption_keys_collections::SESSION_RECORD])?;
        tx.commit().await?;
        Ok(())
    }
}

impl SessionRepository {
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
