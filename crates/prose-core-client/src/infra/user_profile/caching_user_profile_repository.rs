// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;

use crate::app::deps::DynUserProfileService;
use prose_store::prelude::*;

use crate::domain::user_profiles::models::UserProfile;
use crate::domain::user_profiles::repos::UserProfileRepository;
use crate::domain::user_profiles::services::UserProfileService;

#[entity]
pub struct UserProfileRecord {
    id: BareJid,
    payload: UserProfile,
}

pub struct CachingUserProfileRepository {
    store: Store<PlatformDriver>,
    user_profile_service: DynUserProfileService,
}

impl CachingUserProfileRepository {
    pub fn new(store: Store<PlatformDriver>, user_profile_service: DynUserProfileService) -> Self {
        Self {
            store,
            user_profile_service,
        }
    }
}

#[async_trait]
impl UserProfileRepository for CachingUserProfileRepository {
    async fn get(&self, jid: &BareJid) -> Result<Option<UserProfile>> {
        let tx = self
            .store
            .transaction_for_reading(&[UserProfileRecord::collection()])
            .await?;
        let collection = tx.readable_collection(UserProfileRecord::collection())?;
        let record = collection.get::<_, UserProfileRecord>(&jid).await?;

        if let Some(record) = record {
            return Ok(Some(record.payload));
        };

        let Some(profile) = self.user_profile_service.load_profile(&jid).await? else {
            return Ok(None);
        };

        self.set(jid, &profile).await?;
        Ok(Some(profile))
    }

    async fn set(&self, jid: &BareJid, profile: &UserProfile) -> Result<()> {
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

    async fn delete(&self, jid: &BareJid) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserProfileRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(UserProfileRecord::collection())?;
        collection.delete(jid)?;
        tx.commit().await?;
        Ok(())
    }
}
