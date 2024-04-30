// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashSet;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::Mutex;

use prose_store::prelude::*;

use crate::app::deps::DynUserProfileService;
use crate::domain::shared::models::UserId;
use crate::domain::user_profiles::models::UserProfile;
use crate::domain::user_profiles::repos::UserProfileRepository;

#[entity]
pub struct UserProfileRecord {
    id: UserId,
    payload: UserProfile,
}

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
    async fn get(&self, user_id: &UserId) -> Result<Option<UserProfile>> {
        if self.requested_user_profiles.lock().insert(user_id.clone()) {
            return self.load_and_cache_user_profile(user_id).await;
        }
        self.load_cached_user_profile(user_id).await
    }

    async fn set(&self, jid: &UserId, profile: &UserProfile) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserProfileRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(UserProfileRecord::collection())?;
        collection.put_entity(&UserProfileRecord {
            id: jid.clone(),
            payload: profile.clone(),
        })?;
        tx.commit().await?;
        Ok(())
    }

    async fn delete(&self, jid: &UserId) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserProfileRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(UserProfileRecord::collection())?;
        collection.delete(jid)?;
        tx.commit().await?;
        Ok(())
    }

    async fn get_display_name(&self, jid: &UserId) -> Result<Option<String>> {
        // This is a bit heavy-handed right now to load the full contact. prose-store should
        // support multi-column indexes instead so that we can just pull out the required fields.
        Ok(self
            .get(jid)
            .await?
            .and_then(|p| p.full_name().or(p.nickname)))
    }

    async fn reset_after_reconnect(&self) {
        self.requested_user_profiles.lock().clear();
    }

    async fn clear_cache(&self) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserProfileRecord::collection()])
            .await?;
        tx.truncate_collections(&[UserProfileRecord::collection()])?;
        tx.commit().await?;
        Ok(())
    }
}

impl CachingUserProfileRepository {
    async fn load_cached_user_profile(&self, user_id: &UserId) -> Result<Option<UserProfile>> {
        let tx = self
            .store
            .transaction_for_reading(&[UserProfileRecord::collection()])
            .await?;
        let collection = tx.readable_collection(UserProfileRecord::collection())?;
        let record = collection.get::<_, UserProfileRecord>(&user_id).await?;
        Ok(record.map(|record| record.payload))
    }

    async fn load_and_cache_user_profile(&self, user_id: &UserId) -> Result<Option<UserProfile>> {
        let Some(profile) = self.user_profile_service.load_profile(&user_id).await? else {
            _ = self.delete(user_id).await;
            return Ok(None);
        };
        self.set(user_id, &profile).await?;
        return Ok(Some(profile));
    }
}
