// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;

use crate::app::deps::DynContactsService;
use crate::domain::contacts::models::Contact;
use crate::domain::contacts::repos::ContactsRepository;
use crate::domain::shared::models::UserId;

pub struct CachingContactsRepository {
    service: DynContactsService,
    contacts: RwLock<Option<Vec<Contact>>>,
}

impl CachingContactsRepository {
    pub fn new(service: DynContactsService) -> Self {
        Self {
            service,
            contacts: Default::default(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ContactsRepository for CachingContactsRepository {
    async fn get_all(&self, account_jid: &UserId) -> Result<Vec<Contact>> {
        if let Some(contacts) = self.contacts.read().clone() {
            return Ok(contacts);
        }

        let contacts = self.service.load_contacts(account_jid.as_ref()).await?;
        self.contacts
            .write()
            .replace(contacts.iter().cloned().collect());

        Ok(contacts)
    }

    async fn clear_cache(&self) -> Result<()> {
        self.contacts.write().take();
        Ok(())
    }
}
