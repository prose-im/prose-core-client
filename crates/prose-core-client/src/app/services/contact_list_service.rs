// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use futures::future::join_all;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::*;
use crate::app::dtos::Contact as ContactDTO;
use crate::domain::contacts::models::{Contact, PresenceSubRequest as DomainPresenceSubRequest};
use crate::domain::shared::models::{AccountId, CachePolicy};
use crate::domain::user_info::models::UserInfoOptExt;
use crate::dtos::{Group, PresenceSubRequest, PresenceSubRequestId, UserId};

#[derive(InjectDependencies)]
pub struct ContactListService {
    #[inject]
    contact_list_domain_service: DynContactListDomainService,
    #[inject]
    ctx: DynAppContext,
    #[inject]
    user_info_domain_service: DynUserInfoDomainService,
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
        let domain_requests = self
            .contact_list_domain_service
            .load_presence_sub_requests()
            .await?;

        let requests = join_all(
            domain_requests
                .into_iter()
                .map(|id| self.enrich_presence_sub_request(id)),
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
    /// Potential errors are ignored since the additional data is deemed optional, and we rather
    /// return something than nothing.
    async fn enrich_domain_contact(&self, account: &AccountId, contact: Contact) -> ContactDTO {
        let group = if account.is_same_domain(&contact.id) {
            Group::Team
        } else {
            Group::Other
        };

        let user_info = self
            .user_info_domain_service
            .get_user_info(&contact.id, CachePolicy::ReturnCacheDataElseLoad)
            .await
            .unwrap_or_default()
            .into_user_presence_info_or_fallback(contact.id);

        ContactDTO {
            id: user_info.id,
            name: user_info.name,
            full_name: user_info.full_name,
            avatar: user_info.avatar,
            availability: user_info.availability,
            status: user_info.status,
            group,
            presence_subscription: contact.presence_subscription,
        }
    }

    async fn enrich_presence_sub_request(
        &self,
        request: DomainPresenceSubRequest,
    ) -> PresenceSubRequest {
        let user_info = self
            .user_info_domain_service
            .get_user_info(&request.user_id, CachePolicy::ReturnCacheDataElseLoad)
            .await
            .unwrap_or_default()
            .into_user_basic_info_or_fallback(request.user_id);

        PresenceSubRequest {
            id: user_info.id.clone().into(),
            name: user_info.name,
            user_id: user_info.id,
        }
    }
}
