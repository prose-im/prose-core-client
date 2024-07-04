// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use prose_store::prelude::*;

use crate::domain::shared::models::{AccountId, ParticipantId, ParticipantIdRef};
use crate::domain::user_info::models::UserProfile;
use crate::domain::user_info::repos::UserProfileRepository as UserProfileRepositoryTrait;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileRecord {
    id: String,
    account: AccountId,
    participant_id: ParticipantId,
    payload: UserProfile,
}

impl UserProfileRecord {
    fn new(
        account: &AccountId,
        participant_id: ParticipantIdRef<'_>,
        payload: UserProfile,
    ) -> Self {
        Self {
            id: format!("{account}.{}", participant_id.to_raw_key_string()),
            account: account.clone(),
            participant_id: participant_id.to_owned(),
            payload,
        }
    }
}

mod columns {
    pub const ACCOUNT: &str = "account";
    pub const PARTICIPANT_ID: &str = "participant_id";
}

define_entity!(UserProfileRecord, "user_profile",
    account_idx => { columns: [columns::ACCOUNT], unique: false },
    user_idx => { columns: [columns::ACCOUNT, columns::PARTICIPANT_ID], unique: true }
);

pub struct UserProfileRepository {
    store: Store<PlatformDriver>,
}

impl UserProfileRepository {
    pub fn new(store: Store<PlatformDriver>) -> Self {
        Self { store }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl UserProfileRepositoryTrait for UserProfileRepository {
    async fn get(
        &self,
        account: &AccountId,
        participant_id: ParticipantIdRef<'_>,
    ) -> Result<Option<UserProfile>> {
        let tx = self
            .store
            .transaction_for_reading(&[UserProfileRecord::collection()])
            .await?;
        let collection = tx.readable_collection(UserProfileRecord::collection())?;
        let idx = collection.index(&UserProfileRecord::user_idx())?;
        let record = idx
            .get::<_, UserProfileRecord>(&(account, participant_id))
            .await?;
        Ok(record.map(|record| record.payload))
    }

    async fn set(
        &self,
        account: &AccountId,
        participant_id: ParticipantIdRef<'_>,
        profile: Option<&UserProfile>,
    ) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserProfileRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(UserProfileRecord::collection())?;

        if let Some(profile) = profile {
            collection.put_entity(&UserProfileRecord::new(
                account,
                participant_id,
                profile.clone(),
            ))?;
        } else {
            let idx = collection.index(&UserProfileRecord::user_idx())?;
            idx.delete(&(account, participant_id)).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn reset_before_reconnect(&self, _account: &AccountId) -> Result<()> {
        Ok(())
    }

    async fn clear_cache(&self, account: &AccountId) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserProfileRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(UserProfileRecord::collection())?;
        collection
            .delete_all_in_index(&UserProfileRecord::account_idx(), Query::Only(account))
            .await?;
        tx.commit().await?;
        Ok(())
    }
}
