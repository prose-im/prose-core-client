// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashSet;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use prose_store::prelude::*;

use crate::app::deps::DynUserProfileService;
use crate::domain::shared::models::{AccountId, UserId};
use crate::domain::user_profiles::models::UserProfile;
use crate::domain::user_profiles::repos::UserProfileRepository;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileRecord {
    id: String,
    account: AccountId,
    user_id: UserId,
    payload: UserProfile,
}

impl UserProfileRecord {
    fn new(account: &AccountId, user_id: &UserId, payload: UserProfile) -> Self {
        Self {
            id: format!("{}.{}", account, user_id),
            account: account.clone(),
            user_id: user_id.clone(),
            payload,
        }
    }
}

mod columns {
    pub const ACCOUNT: &str = "account";
    pub const USER_ID: &str = "user_id";
}

define_entity!(UserProfileRecord, "user_profile",
    account_idx => { columns: [columns::ACCOUNT], unique: false },
    user_idx => { columns: [columns::ACCOUNT, columns::USER_ID], unique: true }
);

pub struct CachingUserProfileRepository {
    store: Store<PlatformDriver>,
    user_profile_service: DynUserProfileService,
    requested_user_profiles: Mutex<HashSet<UserId>>,
}

impl CachingUserProfileRepository {
    pub fn new(store: Store<PlatformDriver>, user_profile_service: DynUserProfileService) -> Self {
        Self {
            store,
            user_profile_service,
            requested_user_profiles: Default::default(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl UserProfileRepository for CachingUserProfileRepository {
    async fn get(&self, account: &AccountId, user_id: &UserId) -> Result<Option<UserProfile>> {
        if self.requested_user_profiles.lock().insert(user_id.clone()) {
            return self.load_and_cache_user_profile(account, user_id).await;
        }
        self.load_cached_user_profile(account, user_id).await
    }

    async fn set(
        &self,
        account: &AccountId,
        user_id: &UserId,
        profile: &UserProfile,
    ) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserProfileRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(UserProfileRecord::collection())?;
        collection.put_entity(&UserProfileRecord::new(account, user_id, profile.clone()))?;
        tx.commit().await?;
        Ok(())
    }

    async fn delete(&self, account: &AccountId, user_id: &UserId) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserProfileRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(UserProfileRecord::collection())?;
        let idx = collection.index(&UserProfileRecord::user_idx())?;
        idx.delete(&(account, user_id)).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn get_display_name(
        &self,
        account: &AccountId,
        user_id: &UserId,
    ) -> Result<Option<String>> {
        // This is a bit heavy-handed right now to load the full contact. prose-store should
        // support multi-column indexes instead so that we can just pull out the required fields.
        Ok(self
            .get(account, user_id)
            .await?
            .and_then(|p| p.full_name().or(p.nickname)))
    }

    async fn reset_before_reconnect(&self, _account: &AccountId) -> Result<()> {
        self.requested_user_profiles.lock().clear();
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
        self.requested_user_profiles.lock().clear();
        Ok(())
    }
}

impl CachingUserProfileRepository {
    async fn load_cached_user_profile(
        &self,
        account: &AccountId,
        user_id: &UserId,
    ) -> Result<Option<UserProfile>> {
        let tx = self
            .store
            .transaction_for_reading(&[UserProfileRecord::collection()])
            .await?;
        let collection = tx.readable_collection(UserProfileRecord::collection())?;
        let idx = collection.index(&UserProfileRecord::user_idx())?;
        let record = idx.get::<_, UserProfileRecord>(&(account, user_id)).await?;
        Ok(record.map(|record| record.payload))
    }

    async fn load_and_cache_user_profile(
        &self,
        account: &AccountId,
        user_id: &UserId,
    ) -> Result<Option<UserProfile>> {
        let Some(profile) = self.user_profile_service.load_profile(&user_id).await? else {
            _ = self.delete(account, user_id).await;
            return Ok(None);
        };
        self.set(account, user_id, &profile).await?;
        return Ok(Some(profile));
    }
}
