// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::BareJid;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::*;
use crate::app::dtos::Contact;
use crate::domain::shared::utils::build_contact_name;

#[derive(InjectDependencies)]
pub struct ContactsService {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    contacts_service: DynContactsService,
    #[inject]
    contacts_repo: DynContactsRepository,
    #[inject]
    user_info_repo: DynUserInfoRepository,
    #[inject]
    user_profile_repo: DynUserProfileRepository,
}

impl ContactsService {
    pub async fn load_contacts(&self) -> Result<Vec<Contact>> {
        let domain_contacts = self
            .contacts_repo
            .get_all(&self.ctx.connected_id()?.to_user_id())
            .await?;
        let mut contacts = vec![];

        for domain_contact in domain_contacts {
            let profile = self
                .user_profile_repo
                .get(&domain_contact.id)
                .await?
                .unwrap_or_default();
            let user_info = self
                .user_info_repo
                .get_user_info(&domain_contact.id)
                .await?
                .unwrap_or_default();

            let name = build_contact_name(&domain_contact.id, &profile);

            let contact = Contact {
                id: domain_contact.id,
                name,
                availability: user_info.availability,
                status: user_info.activity,
                group: domain_contact.group,
            };
            contacts.push(contact)
        }

        Ok(contacts)
    }

    pub async fn add_contact(&self, jid: &BareJid) -> Result<()> {
        self.contacts_service.add_contact(jid).await
    }
}
