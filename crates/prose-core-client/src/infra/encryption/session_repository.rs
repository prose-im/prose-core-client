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
use crate::domain::shared::models::AccountId;
use crate::dtos::{DeviceId, IdentityKey, SessionData, UserId};
use crate::infra::encryption::encryption_key_records::SessionRecord;

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
    async fn get_session(
        &self,
        account: &AccountId,
        user_id: &UserId,
        device_id: &DeviceId,
    ) -> Result<Option<Session>> {
        let tx = self
            .store
            .transaction_for_reading(&[SessionRecord::collection()])
            .await?;
        let collection = tx.readable_collection(SessionRecord::collection())?;
        let idx = collection.index(&SessionRecord::device_idx())?;

        let session = idx
            .get::<_, SessionRecord>(&(account, user_id, device_id))
            .await?;

        Ok(session.map(Session::from))
    }

    async fn get_all_sessions(
        &self,
        account: &AccountId,
        user_id: &UserId,
    ) -> Result<Vec<Session>> {
        let tx = self
            .store
            .transaction_for_reading(&[SessionRecord::collection()])
            .await?;
        let collection = tx.readable_collection(SessionRecord::collection())?;
        let idx = collection.index(&SessionRecord::user_idx())?;

        let records = idx
            .get_all_values::<SessionRecord>(
                Query::Only((account, user_id)),
                QueryDirection::Forward,
                None,
            )
            .await?
            .into_iter()
            .map(Session::from)
            .collect();

        Ok(records)
    }

    async fn put_session_data(
        &self,
        account: &AccountId,
        user_id: &UserId,
        device_id: &DeviceId,
        data: SessionData,
    ) -> Result<()> {
        self.upsert_session(account, user_id, device_id, move |session| {
            session.data = Some(data)
        })
        .await?;
        Ok(())
    }

    async fn put_identity(
        &self,
        account: &AccountId,
        user_id: &UserId,
        device_id: &DeviceId,
        identity: IdentityKey,
    ) -> Result<bool> {
        self.upsert_session(account, user_id, device_id, move |session| {
            session.identity = Some(identity)
        })
        .await
    }

    async fn put_active_devices(
        &self,
        account: &AccountId,
        user_id: &UserId,
        device_ids: &[DeviceId],
    ) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[SessionRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(SessionRecord::collection())?;
        let idx = collection.index(&SessionRecord::user_idx())?;

        let mut records = idx
            .get_all_values::<SessionRecord>(
                Query::Only((account, user_id)),
                QueryDirection::Forward,
                None,
            )
            .await?;

        let device_ids = device_ids.into_iter().collect::<HashSet<_>>();

        for record in records.iter_mut() {
            if record.is_active == device_ids.contains(&record.device_id) {
                continue;
            }

            record.is_active = !record.is_active;
            collection.put_entity(record)?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn clear_cache(&self, account: &AccountId) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[SessionRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(SessionRecord::collection())?;
        collection
            .delete_all_in_index(&SessionRecord::account_idx(), Query::Only(account))
            .await?;
        tx.commit().await?;
        Ok(())
    }
}

impl SessionRepository {
    /// The return value represents whether an existing identity was replaced (Ok(true)). If it is
    /// new or hasn't changed, the return value should be Ok(false).
    async fn upsert_session<F: FnOnce(&mut SessionRecord)>(
        &self,
        account: &AccountId,
        user_id: &UserId,
        device_id: &DeviceId,
        handler: F,
    ) -> Result<bool> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[SessionRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(SessionRecord::collection())?;
        let idx = collection.index(&SessionRecord::device_idx())?;

        let existing_session_changed = match idx
            .get::<_, SessionRecord>(&(account, user_id, device_id))
            .await?
        {
            Some(session) => {
                let mut updated_session = session.clone();
                handler(&mut updated_session);

                if updated_session != session {
                    collection.put_entity(&updated_session)?;
                    tx.commit().await?;
                    true
                } else {
                    false
                }
            }
            None => {
                let session = Session {
                    user_id: user_id.clone(),
                    device_id: device_id.clone(),
                    trust: Trust::Undecided,
                    is_active: true,
                    identity: None,
                    data: None,
                };
                let mut record = SessionRecord::new(account, session);
                handler(&mut record);
                collection.put_entity(&record)?;
                tx.commit().await?;
                false
            }
        };

        Ok(existing_session_changed)
    }
}
