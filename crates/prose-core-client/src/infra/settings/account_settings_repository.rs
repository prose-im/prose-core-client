// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use prose_store::prelude::*;

use crate::domain::settings::models::AccountSettings;
use crate::domain::settings::repos::AccountSettingsRepository as DomainAccountSettingsRepository;
use crate::domain::shared::models::AccountId;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountSettingsRecord {
    id: AccountId,
    payload: AccountSettings,
}

define_entity!(AccountSettingsRecord, "account_settings", AccountId);

pub struct AccountSettingsRepository {
    store: Store<PlatformDriver>,
}

impl AccountSettingsRepository {
    pub fn new(store: Store<PlatformDriver>) -> Self {
        Self { store }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl DomainAccountSettingsRepository for AccountSettingsRepository {
    async fn get(&self, account: &AccountId) -> Result<AccountSettings> {
        let tx = self
            .store
            .transaction_for_reading(&[AccountSettingsRecord::collection()])
            .await?;
        let collection = tx.readable_collection(AccountSettingsRecord::collection())?;
        let settings = collection.get::<_, AccountSettingsRecord>(account).await?;
        Ok(settings.map(|s| s.payload).unwrap_or_default())
    }

    async fn update(
        &self,
        account: &AccountId,
        block: Box<dyn for<'a> FnOnce(&'a mut AccountSettings) + Send>,
    ) -> Result<()> {
        upsert!(
            AccountSettingsRecord,
            store: self.store,
            id: account,
            insert_if_needed: || AccountSettingsRecord {
                id: account.clone(),
                payload: Default::default()
            },
            update: |settings: &mut AccountSettingsRecord| block(&mut settings.payload)
        );
        Ok(())
    }

    async fn clear_cache(&self, account: &AccountId) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[AccountSettingsRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(AccountSettingsRecord::collection())?;
        collection.delete(account).await?;
        tx.commit().await?;
        Ok(())
    }
}
