// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::BareJid;
use prose_proc_macros::InjectDependencies;

use crate::app::deps::*;
use crate::app::dtos::Contact;
use crate::util::concatenate_names;
use crate::util::jid_ext::JidExt;

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
            .get_all(&self.ctx.connected_jid()?.into_bare())
            .await?;
        let mut contacts = vec![];

        for domain_contact in domain_contacts {
            let profile = self
                .user_profile_repo
                .get(&domain_contact.jid)
                .await?
                .unwrap_or_default();
            let user_info = self
                .user_info_repo
                .get_user_info(&domain_contact.jid)
                .await?
                .unwrap_or_default();

            let name = concatenate_names(&profile.first_name, &profile.last_name)
                .or(profile.nickname)
                .or(domain_contact.name)
                .unwrap_or(domain_contact.jid.to_display_name());

            let contact = Contact {
                jid: domain_contact.jid,
                name,
                availability: user_info.availability,
                activity: user_info.activity,
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
