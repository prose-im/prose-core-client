// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use futures::future::join_all;
use futures::join;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::*;
use crate::app::dtos::Contact as ContactDTO;
use crate::domain::contacts::models::Contact;
use crate::domain::shared::models::AccountId;
use crate::domain::shared::utils::build_contact_name;
use crate::dtos::{Group, PresenceSubRequest, PresenceSubRequestId, UserId};

#[derive(InjectDependencies)]
pub struct ContactListService {
    #[inject]
    contact_list_domain_service: DynContactListDomainService,
    #[inject]
    ctx: DynAppContext,
    #[inject]
    user_info_repo: DynUserInfoRepository,
    #[inject]
    user_profile_repo: DynUserProfileRepository,
}

impl ContactListService {
    pub async fn load_contacts(&self) -> Result<Vec<ContactDTO>> {
        let account = self.ctx.connected_account()?;
        let domain_contacts = self.contact_list_domain_service.load_contacts().await?;

        let contacts = join_all(
            domain_contacts
                .into_iter()
                .map(|c| self.enrich_domain_contact(&account, c)),
        )
        .await;

        Ok(contacts)
    }

    pub async fn add_contact(&self, jid: &UserId) -> Result<()> {
        self.contact_list_domain_service.add_contact(jid).await?;
        Ok(())
    }

    pub async fn remove_contact(&self, jid: &UserId) -> Result<()> {
        self.contact_list_domain_service.remove_contact(jid).await?;
        Ok(())
    }

    pub async fn request_presence_sub(&self, from: &UserId) -> Result<()> {
        self.contact_list_domain_service
            .request_presence_sub(from)
            .await
    }

    pub async fn load_presence_sub_requests(&self) -> Result<Vec<PresenceSubRequest>> {
        let account = self.ctx.connected_account()?;
        let requesting_user_ids = self
            .contact_list_domain_service
            .load_presence_sub_requests()
            .await?;

        let requests = join_all(
            requesting_user_ids
                .into_iter()
                .map(|id| self.enrich_presence_sub_request(&account, id)),
        )
        .await;

        Ok(requests)
    }

    pub async fn approve_presence_sub_request(&self, id: &PresenceSubRequestId) -> Result<()> {
        self.contact_list_domain_service
            .approve_presence_sub_request(&id.to_user_id())
            .await?;
        Ok(())
    }

    pub async fn deny_presence_sub_request(&self, id: &PresenceSubRequestId) -> Result<()> {
        self.contact_list_domain_service
            .deny_presence_sub_request(&id.to_user_id())
            .await?;
        Ok(())
    }
}

impl ContactListService {
    /// Converts a domain `Contact` to a `Contact` DTO by enriching it with additional data.
    /// Potential errors are ignored since the additional data is deemed optional and we rather
    /// return something than nothing.
    async fn enrich_domain_contact(&self, account: &AccountId, contact: Contact) -> ContactDTO {
        let (profile, user_info) = join!(
            self.user_profile_repo.get(account, &contact.id),
            self.user_info_repo.get_user_info(account, &contact.id)
        );

        // We'll march on even in the event of a failure…
        let profile = profile.unwrap_or_default().unwrap_or_default();
        let user_info = user_info.unwrap_or_default().unwrap_or_default();
        let name = build_contact_name(&contact.id, &profile);
        let group = if account.is_same_domain(&contact.id) {
            Group::Team
        } else {
            Group::Other
        };

        ContactDTO {
            id: contact.id,
            name,
            availability: user_info.availability,
            status: user_info.activity,
            group,
            presence_subscription: contact.presence_subscription,
        }
    }

    async fn enrich_presence_sub_request(
        &self,
        account: &AccountId,
        user_id: UserId,
    ) -> PresenceSubRequest {
        let profile = self
            .user_profile_repo
            .get(account, &user_id)
            .await
            .unwrap_or_default()
            .unwrap_or_default();
        let name = build_contact_name(&user_id, &profile);
        PresenceSubRequest {
            id: user_id.clone().into(),
            name,
            user_id,
        }
    }
}
