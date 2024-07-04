// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;

use crate::app::deps::DynContactListService;
use crate::domain::contacts::models::{Contact, PresenceSubscription};
use crate::domain::contacts::repos::ContactListRepository;
use crate::domain::shared::models::{AccountId, UserId};

pub struct CachingContactsRepository {
    service: DynContactListService,
    contacts: RwLock<Option<Vec<Contact>>>,
}

impl CachingContactsRepository {
    pub fn new(service: DynContactListService) -> Self {
        Self {
            service,
            contacts: Default::default(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ContactListRepository for CachingContactsRepository {
    async fn get_all(&self, _account: &AccountId) -> Result<Vec<Contact>> {
        self.load_contacts_if_needed().await?;
        Ok(self.contacts.read().clone().unwrap_or_else(|| vec![]))
    }

    async fn set(
        &self,
        _account: &AccountId,
        contact_id: &UserId,
        subscription: PresenceSubscription,
    ) -> Result<bool> {
        self.load_contacts_if_needed().await?;

        let mut guard = self.contacts.write();
        let contacts = guard.get_or_insert_with(Default::default);

        if let Some(contact) = contacts.iter_mut().find(|c| &c.id == contact_id) {
            if contact.presence_subscription == subscription {
                return Ok(false);
            }
            contact.presence_subscription = subscription;
            return Ok(true);
        };

        contacts.push(Contact {
            id: contact_id.clone(),
            name: Some(contact_id.formatted_username()),
            presence_subscription: subscription,
        });

        Ok(true)
    }

    async fn delete(&self, _account: &AccountId, contact_id: &UserId) -> Result<bool> {
        self.load_contacts_if_needed().await?;

        let mut guard = self.contacts.write();
        let contacts = guard.get_or_insert_with(Default::default);

        let Some(idx) = contacts.iter().position(|c| &c.id == contact_id) else {
            return Ok(false);
        };

        contacts.swap_remove(idx);

        Ok(true)
    }

    async fn reset_before_reconnect(&self, account: &AccountId) -> Result<()> {
        self.clear_cache(account).await
    }

    async fn clear_cache(&self, _account: &AccountId) -> Result<()> {
        self.contacts.write().take();
        Ok(())
    }
}

impl CachingContactsRepository {
    async fn load_contacts_if_needed(&self) -> Result<()> {
        if self.contacts.read().is_some() {
            return Ok(());
        }

        let contacts = self.service.load_contacts().await?;
        self.contacts.write().replace(contacts);
        Ok(())
    }
}
