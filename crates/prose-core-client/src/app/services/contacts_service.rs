// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use futures::future::join_all;
use futures::join;
use jid::BareJid;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::*;
use crate::app::dtos::Contact as ContactDTO;
use crate::domain::contacts::models::Contact;
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
    pub async fn load_contacts(&self) -> Result<Vec<ContactDTO>> {
        let domain_contacts = self
            .contacts_repo
            .get_all(&self.ctx.connected_id()?.to_user_id())
            .await?;

        let contacts = join_all(
            domain_contacts
                .into_iter()
                .map(|c| self.enrich_domain_contact(c)),
        )
        .await;

        Ok(contacts)
    }

    pub async fn add_contact(&self, jid: &BareJid) -> Result<()> {
        self.contacts_service.add_contact(jid).await
    }

    pub async fn remove_contact(&self, jid: &BareJid) -> Result<()> {
        self.contacts_service.remove_contact(jid).await
    }
}

impl ContactsService {
    /// Converts a domain `Contact` to a `Contact` DTO by enriching it with additional data.
    /// Potential errors are ignored since the additional data is deemed optional and we rather
    /// return something than nothing.
    async fn enrich_domain_contact(&self, contact: Contact) -> ContactDTO {
        let (profile, user_info) = join!(
            self.user_profile_repo.get(&contact.id),
            self.user_info_repo.get_user_info(&contact.id)
        );

        // We'll march on even in the event of a failure…
        let profile = profile.unwrap_or_default().unwrap_or_default();
        let user_info = user_info.unwrap_or_default().unwrap_or_default();
        let name = build_contact_name(&contact.id, &profile);

        ContactDTO {
            id: contact.id,
            name,
            availability: user_info.availability,
            status: user_info.activity,
            group: contact.group,
        }
    }
}
