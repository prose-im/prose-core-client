// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use jid::BareJid;

use prose_store::prelude::*;
use prose_store::Database;

use crate::domain::messaging::repos::DraftsRepository as DomainDraftsRepository;

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
    pub id: BareJid,
    pub text: String,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl DomainDraftsRepository for DraftsRepository {
    async fn get(&self, room_id: &BareJid) -> anyhow::Result<Option<String>> {
        let tx = self
            .store
            .transaction_for_reading(&[DraftsRecord::collection()])
            .await?;
        let collection = tx.readable_collection(DraftsRecord::collection())?;
        let record = collection.get::<_, DraftsRecord>(room_id).await?;
        Ok(record.map(|r| r.text))
    }

    async fn set(&self, room_id: &BareJid, draft: Option<&str>) -> anyhow::Result<()> {
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
}
