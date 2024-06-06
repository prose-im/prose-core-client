// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use prose_store::prelude::*;

use crate::domain::messaging::repos::DraftsRepository as DraftsRepositoryTrait;
use crate::domain::shared::models::AccountId;
use crate::dtos::RoomId;

#[derive(Serialize, Deserialize)]
pub struct DraftsRecord {
    id: String,
    account: AccountId,
    room_id: RoomId,
    text: String,
}

impl DraftsRecord {
    fn new(account: &AccountId, room_id: &RoomId, text: String) -> Self {
        Self {
            id: format!("{}.{}", account, room_id),
            account: account.clone(),
            room_id: room_id.clone(),
            text,
        }
    }
}

mod columns {
    pub const ACCOUNT: &str = "account";
    pub const ROOM_ID: &str = "room_id";
}

define_entity!(DraftsRecord, "drafts",
    account_idx => { columns: [columns::ACCOUNT], unique: false },
    room_idx => { columns: [columns::ACCOUNT, columns::ROOM_ID], unique: true }
);

pub struct DraftsRepository {
    store: Store<PlatformDriver>,
}

impl DraftsRepository {
    pub fn new(store: Store<PlatformDriver>) -> Self {
        Self { store }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl DraftsRepositoryTrait for DraftsRepository {
    async fn get(&self, account: &AccountId, room_id: &RoomId) -> Result<Option<String>> {
        let tx = self
            .store
            .transaction_for_reading(&[DraftsRecord::collection()])
            .await?;
        let collection = tx.readable_collection(DraftsRecord::collection())?;
        let idx = collection.index(&DraftsRecord::room_idx())?;
        let record = idx.get::<_, DraftsRecord>(&(account, room_id)).await?;
        Ok(record.and_then(|r| (!r.text.is_empty()).then_some(r.text)))
    }

    async fn set(&self, account: &AccountId, room_id: &RoomId, draft: Option<&str>) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[DraftsRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(DraftsRecord::collection())?;

        match draft {
            Some(draft) if !draft.is_empty() => {
                collection.put_entity(&DraftsRecord::new(account, room_id, draft.to_string()))?;
            }
            _ => {
                let idx = collection.index(&DraftsRecord::room_idx())?;
                idx.delete(&(account, room_id)).await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }

    async fn clear_cache(&self, account: &AccountId) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[DraftsRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(DraftsRecord::collection())?;
        collection
            .delete_all_in_index(&DraftsRecord::account_idx(), Query::Only(account))
            .await?;
        tx.commit().await?;
        Ok(())
    }
}
