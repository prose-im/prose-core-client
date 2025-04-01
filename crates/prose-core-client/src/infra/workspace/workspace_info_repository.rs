// prose-core-client/prose-core-client
//
// Copyright: 2025, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::AccountId;
use crate::domain::workspace::models::WorkspaceInfo;
use crate::domain::workspace::repos::{
    UpdateHandler, WorkspaceInfoRepository as WorkspaceInfoRepositoryTrait,
};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use prose_store::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceInfoRecord {
    id: String,
    account: AccountId,
    payload: WorkspaceInfo,
}

mod columns {
    pub const ACCOUNT: &str = "account";
}

define_entity!(WorkspaceInfoRecord, "workspace_info",
    account_idx => { columns: [columns::ACCOUNT], unique: true }
);

pub struct WorkspaceInfoRepository {
    store: Store<PlatformDriver>,
}

impl WorkspaceInfoRepository {
    pub fn new(store: Store<PlatformDriver>) -> Self {
        Self { store }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl WorkspaceInfoRepositoryTrait for WorkspaceInfoRepository {
    async fn get(&self, account: &AccountId) -> Result<Option<WorkspaceInfo>> {
        let tx = self
            .store
            .transaction_for_reading(&[WorkspaceInfoRecord::collection()])
            .await?;
        let collection = tx.readable_collection(WorkspaceInfoRecord::collection())?;
        let record = collection.get::<_, WorkspaceInfoRecord>(account).await?;
        Ok(record.map(|record| record.payload))
    }

    // Upserts `WorkspaceInfo`. Returns `true` if the `WorkspaceInfo` was changed
    // after executing `handler`.
    async fn update(&self, account: &AccountId, handler: UpdateHandler) -> Result<bool> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[WorkspaceInfoRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(WorkspaceInfoRecord::collection())?;
        let mut record = collection
            .get::<_, WorkspaceInfoRecord>(account)
            .await?
            .unwrap_or_else(|| WorkspaceInfoRecord {
                id: account.to_string(),
                account: account.clone(),
                payload: WorkspaceInfo::default(),
            });

        let old_payload = record.payload.clone();
        handler(&mut record.payload);

        if record.payload == old_payload {
            return Ok(false);
        }

        collection.put_entity(&record)?;
        tx.commit().await?;

        Ok(true)
    }

    async fn clear_cache(&self, account: &AccountId) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[WorkspaceInfoRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(WorkspaceInfoRecord::collection())?;
        collection.delete(account).await?;
        tx.commit().await?;
        Ok(())
    }
}
