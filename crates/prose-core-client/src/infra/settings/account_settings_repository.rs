// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;

use prose_store::prelude::*;
use prose_store::Database;

use crate::domain::settings::models::AccountSettings;
use crate::domain::settings::repos::AccountSettingsRepository as DomainAccountSettingsRepository;

#[entity]
pub struct AccountSettingsRecord {
    id: BareJid,
    payload: AccountSettings,
}

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
    async fn get(&self, jid: &BareJid) -> Result<AccountSettings> {
        let tx = self
            .store
            .transaction_for_reading(&[AccountSettingsRecord::collection()])
            .await?;
        let collection = tx.readable_collection(AccountSettingsRecord::collection())?;
        let settings = collection.get::<_, AccountSettingsRecord>(jid).await?;
        Ok(settings.map(|s| s.payload).unwrap_or_default())
    }

    async fn update(
        &self,
        jid: &BareJid,
        block: Box<dyn for<'a> FnOnce(&'a mut AccountSettings) + Send>,
    ) -> Result<()> {
        upsert!(
            AccountSettingsRecord,
            store: self.store,
            id: jid,
            insert_if_needed: || AccountSettingsRecord {
                id: jid.clone(),
                payload: Default::default()
            },
            update: |settings: &mut AccountSettingsRecord| block(&mut settings.payload)
        );
        Ok(())
    }
}
