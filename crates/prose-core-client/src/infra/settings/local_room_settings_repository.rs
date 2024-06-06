// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use prose_store::prelude::{Entity, PlatformDriver, Store};
use prose_store::{
    define_entity, Database, IndexSpec, IndexedCollection, Query, ReadTransaction,
    ReadableCollection, WritableCollection, WriteTransaction,
};

use crate::domain::settings::models::LocalRoomSettings;
use crate::domain::settings::repos::LocalRoomSettingsRepository as LocalRoomSettingsRepositoryTrait;
use crate::domain::shared::models::AccountId;
use crate::dtos::RoomId;

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalRoomSettingsRecord {
    id: String,
    account: AccountId,
    room_id: RoomId,
    payload: LocalRoomSettings,
}

mod columns {
    pub const ACCOUNT: &str = "account";
    pub const ROOM_ID: &str = "room_id";
}

define_entity!(LocalRoomSettingsRecord, "room_settings_local",
    account_idx => { columns: [columns::ACCOUNT], unique: false },
    room_idx => { columns: [columns::ACCOUNT, columns::ROOM_ID], unique: true }
);

pub struct LocalRoomSettingsRepository {
    store: Store<PlatformDriver>,
}

impl LocalRoomSettingsRepository {
    pub fn new(store: Store<PlatformDriver>) -> Self {
        Self { store }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl LocalRoomSettingsRepositoryTrait for LocalRoomSettingsRepository {
    async fn get(&self, account: &AccountId, room_id: &RoomId) -> Result<LocalRoomSettings> {
        let tx = self
            .store
            .transaction_for_reading(&[LocalRoomSettingsRecord::collection()])
            .await?;
        let collection = tx.readable_collection(LocalRoomSettingsRecord::collection())?;
        let idx = collection.index(&LocalRoomSettingsRecord::room_idx())?;
        let settings = idx
            .get::<_, LocalRoomSettingsRecord>(&(account, room_id))
            .await?;
        Ok(settings.map(|s| s.payload).unwrap_or_default())
    }

    async fn update(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        block: Box<dyn for<'a> FnOnce(&'a mut LocalRoomSettings) + Send>,
    ) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[LocalRoomSettingsRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(LocalRoomSettingsRecord::collection())?;
        let idx = collection.index(&LocalRoomSettingsRecord::room_idx())?;
        let mut settings = idx
            .get::<_, LocalRoomSettingsRecord>(&(account, room_id))
            .await?
            .map(|s| s.payload)
            .unwrap_or_default();
        block(&mut settings);
        collection.put_entity(&LocalRoomSettingsRecord {
            id: format!("{}-{}", account, room_id.to_raw_key_string()),
            account: account.clone(),
            room_id: room_id.clone(),
            payload: settings,
        })?;
        tx.commit().await?;
        Ok(())
    }

    async fn clear_cache(&self, account: &AccountId) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[LocalRoomSettingsRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(LocalRoomSettingsRecord::collection())?;
        collection
            .delete_all_in_index(
                &LocalRoomSettingsRecord::account_idx(),
                Query::Only(account),
            )
            .await?;
        tx.commit().await?;
        Ok(())
    }
}
