// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;

use prose_store::prelude::*;

use crate::app::deps::DynUserInfoService;
use crate::domain::shared::models::{UserId, UserOrResourceId, UserResourceId};
use crate::domain::user_info::models::{AvatarMetadata, Presence, UserInfo, UserStatus};
use crate::domain::user_info::repos::UserInfoRepository;

use super::PresenceMap;

#[entity]
pub struct UserInfoRecord {
    id: UserId,
    payload: UserInfo,
}

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
    fn resolve_user_id_to_user_resource_id(&self, jid: &UserId) -> Option<UserResourceId> {
        let presences = self.presences.read();
        let Some(resource) = presences
            .get_highest_presence(jid)
            .and_then(|entry| entry.resource.as_deref())
        else {
            return None;
        };

        Some(jid.with_resource(resource).expect("Invalid resource"))
    }

    async fn get_user_info(&self, jid: &UserId) -> Result<Option<UserInfo>> {
        let tx = self
            .store
            .transaction_for_reading(&[UserInfoRecord::collection()])
            .await?;
        let collection = tx.readable_collection(UserInfoRecord::collection())?;
        let mut record = collection
            .get::<_, UserInfoRecord>(jid)
            .await?
            .unwrap_or_else(|| UserInfoRecord {
                id: jid.clone(),
                payload: Default::default(),
            });

        let presence = self
            .presences
            .read()
            .get_highest_presence(jid)
            .map(|entry| entry.presence.clone())
            .unwrap_or_default();

        record.payload.availability = presence.availability;

        if record.payload.avatar.is_none() {
            record.payload.avatar = self
                .user_info_service
                .load_latest_avatar_metadata(jid)
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

    async fn set_avatar_metadata(&self, jid: &UserId, metadata: &AvatarMetadata) -> Result<()> {
        upsert!(
            UserInfoRecord,
            store: self.store,
            id: jid,
            insert_if_needed: || UserInfoRecord {
                id: jid.clone(),
                payload: Default::default(),
            },
            update: move |record: &mut UserInfoRecord| record.payload.avatar = Some(metadata.to_info())
        );
        Ok(())
    }

    async fn set_user_activity(
        &self,
        jid: &UserId,
        user_activity: Option<&UserStatus>,
    ) -> Result<()> {
        upsert!(
            UserInfoRecord,
            store: self.store,
            id: jid,
            insert_if_needed: || UserInfoRecord {
                id: jid.clone(),
                payload: Default::default(),
            },
            update: move |record: &mut UserInfoRecord| record.payload.activity = user_activity.cloned()
        );
        Ok(())
    }

    async fn set_user_presence(&self, jid: &UserOrResourceId, presence: &Presence) -> Result<()> {
        let mut map = self.presences.write();
        map.update_presence(jid, presence.clone().into());
        Ok(())
    }

    async fn clear_cache(&self) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[UserInfoRecord::collection()])
            .await?;
        tx.truncate_collections(&[UserInfoRecord::collection()])?;
        tx.commit().await?;
        self.presences.write().clear();
        Ok(())
    }
}
