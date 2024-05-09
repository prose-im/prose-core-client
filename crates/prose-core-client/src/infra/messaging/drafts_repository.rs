// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_store::prelude::*;

use crate::domain::messaging::repos::DraftsRepository as DraftsRepositoryTrait;
use crate::dtos::RoomId;

pub struct DraftsRepository {
    store: Store<PlatformDriver>,
}

impl DraftsRepository {
    pub fn new(store: Store<PlatformDriver>) -> Self {
        Self { store }
    }
}

#[entity]
pub struct DraftsRecord {
    pub id: RoomId,
    pub text: String,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl DraftsRepositoryTrait for DraftsRepository {
    async fn get(&self, room_id: &RoomId) -> Result<Option<String>> {
        let tx = self
            .store
            .transaction_for_reading(&[DraftsRecord::collection()])
            .await?;
        let collection = tx.readable_collection(DraftsRecord::collection())?;
        let record = collection.get::<_, DraftsRecord>(room_id).await?;
        Ok(record.and_then(|r| (!r.text.is_empty()).then_some(r.text)))
    }

    async fn set(&self, room_id: &RoomId, draft: Option<&str>) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[DraftsRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(DraftsRecord::collection())?;

        if let Some(draft) = draft {
            collection.put_entity(&DraftsRecord {
                id: room_id.clone(),
                text: draft.to_string(),
            })?;
        } else {
            collection.delete(room_id)?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn clear_cache(&self) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[DraftsRecord::collection()])
            .await?;
        tx.truncate_collections(&[DraftsRecord::collection()])?;
        tx.commit().await?;
        Ok(())
    }
}
