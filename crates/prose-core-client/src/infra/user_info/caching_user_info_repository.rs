// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use prose_store::prelude::*;

use crate::app::deps::DynUserInfoService;
use crate::domain::shared::models::{AccountId, UserId, UserOrResourceId, UserResourceId};
use crate::domain::user_info::models::{AvatarMetadata, Presence, UserInfo, UserStatus};
use crate::domain::user_info::repos::UserInfoRepository;

use super::PresenceMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfoRecord {
    id: String,
    account: AccountId,
    user_id: UserId,
    payload: UserInfo,
}

impl UserInfoRecord {
    fn new(account: &AccountId, user_id: &UserId, payload: UserInfo) -> Self {
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

define_entity!(UserInfoRecord, "user_info",
    account_idx => { columns: [columns::ACCOUNT], unique: false },
    user_idx => { columns: [columns::ACCOUNT, columns::USER_ID], unique: true }
);

pub struct CachingUserInfoRepository {
    store: Store<PlatformDriver>,
    user_info_service: DynUserInfoService,
    presences: RwLock<PresenceMap>,
}

impl CachingUserInfoRepository {
    pub fn new(store: Store<PlatformDriver>, user_info_service: DynUserInfoService) -> Self {
        Self {
            store,
            user_info_service,
            presences: Default::default(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl UserInfoRepository for CachingUserInfoRepository {
    fn resolve_user_id_to_user_resource_id(
        &self,
        _account: &AccountId,
        user_id: &UserId,
    ) -> Option<UserResourceId> {
        let presences = self.presences.read();
        let Some(resource) = presences
            .get_highest_presence(user_id)
            .and_then(|entry| entry.resource.as_deref())
        else {
            return None;
        };

        Some(user_id.with_resource(resource).expect("Invalid resource"))
    }

    async fn get_user_info(
        &self,
        account: &AccountId,
        user_id: &UserId,
    ) -> Result<Option<UserInfo>> {
        let tx = self
            .store
            .transaction_for_reading(&[UserInfoRecord::collection()])
            .await?;
        let collection = tx.readable_collection(UserInfoRecord::collection())?;
        let idx = collection.index(&UserInfoRecord::user_idx())?;
        let mut record = idx
            .get::<_, UserInfoRecord>(&(account, user_id))
            .await?
            .unwrap_or_else(|| UserInfoRecord::new(account, user_id, Default::default()));

        let presence = self
            .presences
            .read()
            .get_highest_presence(user_id)
            .map(|entry| entry.presence.clone())
            .unwrap_or_default();

        record.payload.availability = presence.availability;

        if record.payload.avatar.is_none() {
            record.payload.avatar = self
                .user_info_service
                .load_latest_avatar_metadata(user_id)
                .await?
                .map(|metadata| metadata.into_info());

            if record.payload.avatar.is_some() {
                let tx = self
                    .store
                    .transaction_for_reading_and_writing(&[UserInfoRecord::collection()])
                    .await?;
                let collection = tx.writeable_collection(UserInfoRecord::collection())?;
                collection.put_entity(&record)?;
                tx.commit().await?;
            }
        }

        Ok(Some(record.payload))
    }

    async fn set_avatar_metadata(
        &self,
        account: &AccountId,
        user_id: &UserId,
        metadata: &AvatarMetadata,
    ) -> Result<()> {
        self.upsert_user_info(account, user_id, |record| {
            record.payload.avatar = Some(metadata.to_info())
        })
        .await
    }

    async fn set_user_activity(
        &self,
        account: &AccountId,
        user_id: &UserId,
        user_activity: Option<&UserStatus>,
    ) -> Result<()> {
        self.upsert_user_info(account, user_id, |record| {
            record.payload.activity = user_activity.cloned()
        })
        .await
    }

    async fn set_user_presence(
        &self,
        _account: &AccountId,
        user_id: &UserOrResourceId,
        presence: &Presence,
    ) -> Result<()> {
        let mut map = self.presences.write();
        map.update_presence(user_id, presence.clone().into());
        Ok(())
    }

    async fn clear_cache(&self, account: &AccountId) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserInfoRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(UserInfoRecord::collection())?;
        collection
            .delete_all_in_index(&UserInfoRecord::account_idx(), Query::Only(account))
            .await?;
        tx.commit().await?;
        self.presences.write().clear();
        Ok(())
    }
}

impl CachingUserInfoRepository {
    async fn upsert_user_info<F: FnOnce(&mut UserInfoRecord)>(
        &self,
        account: &AccountId,
        user_id: &UserId,
        handler: F,
    ) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserInfoRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(UserInfoRecord::collection())?;
        let idx = collection.index(&UserInfoRecord::user_idx())?;
        let mut record = idx
            .get::<_, UserInfoRecord>(&(account, user_id))
            .await?
            .unwrap_or_else(|| UserInfoRecord::new(account, user_id, Default::default()));
        handler(&mut record);
        collection.put_entity(&record)?;
        tx.commit().await?;
        Ok(())
    }
}
