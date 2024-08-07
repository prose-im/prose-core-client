// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_proc_macros::DependenciesStruct;

use crate::app::deps::{
    DynAppContext, DynClientEventDispatcher, DynContactListRepository, DynContactListService,
    DynPresenceSubRequestsRepository,
};
use crate::domain::contacts::models::{Contact, PresenceSubRequest, PresenceSubscription};
use crate::dtos::UserId;
use crate::ClientEvent;

use super::super::ContactListDomainService as ContactListDomainServiceTrait;

#[derive(DependenciesStruct)]
pub struct ContactListDomainService {
    ctx: DynAppContext,
    client_event_dispatcher: DynClientEventDispatcher,
    contact_list_repo: DynContactListRepository,
    contact_list_service: DynContactListService,
    presence_sub_requests_repo: DynPresenceSubRequestsRepository,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ContactListDomainServiceTrait for ContactListDomainService {
    async fn load_contacts(&self) -> Result<Vec<Contact>> {
        self.contact_list_repo
            .get_all(&self.ctx.connected_account()?)
            .await
    }

    async fn add_contact(&self, user_id: &UserId) -> Result<()> {
        self.contact_list_service.add_contact(user_id).await?;
        self.contact_list_service
            .preapprove_subscription_request(user_id)
            .await?;
        self.contact_list_service
            .subscribe_to_presence(user_id)
            .await?;

        if self
            .contact_list_repo
            .set(
                &self.ctx.connected_account()?,
                user_id,
                PresenceSubscription::Requested,
            )
            .await?
        {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::ContactListChanged);
        }

        Ok(())
    }

    async fn remove_contact(&self, user_id: &UserId) -> Result<()> {
        self.contact_list_service.remove_contact(user_id).await?;
        self.contact_list_service
            .revoke_presence_subscription(user_id)
            .await?;

        if self
            .contact_list_repo
            .delete(&self.ctx.connected_account()?, user_id)
            .await?
        {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::ContactListChanged);
        }

        Ok(())
    }

    async fn request_presence_sub(&self, from: &UserId) -> Result<()> {
        self.contact_list_service
            .subscribe_to_presence(from)
            .await?;
        Ok(())
    }

    async fn load_presence_sub_requests(&self) -> Result<Vec<PresenceSubRequest>> {
        self.presence_sub_requests_repo
            .get_all(&self.ctx.connected_account()?)
            .await
    }

    async fn approve_presence_sub_request(&self, from: &UserId) -> Result<()> {
        self.contact_list_service
            .approve_presence_sub_request(from)
            .await?;

        if self
            .presence_sub_requests_repo
            .delete(&self.ctx.connected_account()?, from)
            .await?
        {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::PresenceSubRequestsChanged);
        }

        self.add_contact(from).await?;

        Ok(())
    }

    async fn deny_presence_sub_request(&self, from: &UserId) -> Result<()> {
        self.contact_list_service
            .deny_presence_sub_request(from)
            .await?;

        if self
            .presence_sub_requests_repo
            .delete(&self.ctx.connected_account()?, from)
            .await?
        {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::PresenceSubRequestsChanged);
        }

        Ok(())
    }

    async fn handle_updated_contact(
        &self,
        user_id: &UserId,
        subscription: PresenceSubscription,
    ) -> Result<()> {
        if self
            .contact_list_repo
            .set(&self.ctx.connected_account()?, user_id, subscription)
            .await?
        {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::ContactListChanged);
        }
        Ok(())
    }

    async fn handle_removed_contact(&self, user_id: &UserId) -> Result<()> {
        if self
            .contact_list_repo
            .delete(&self.ctx.connected_account()?, user_id)
            .await?
        {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::ContactListChanged);
        }
        Ok(())
    }

    async fn handle_presence_sub_request(
        &self,
        from: &UserId,
        nickname: Option<String>,
    ) -> Result<()> {
        if self
            .presence_sub_requests_repo
            .set(
                &self.ctx.connected_account()?,
                PresenceSubRequest {
                    user_id: from.clone(),
                    name: nickname,
                },
            )
            .await?
        {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::PresenceSubRequestsChanged);
        }
        Ok(())
    }

    async fn reset_before_reconnect(&self) -> Result<()> {
        let account = self.ctx.connected_account()?;
        self.contact_list_repo
            .reset_before_reconnect(&account)
            .await?;
        Ok(())
    }

    async fn clear_cache(&self) -> Result<()> {
        let account = self.ctx.connected_account()?;
        self.contact_list_repo.clear_cache(&account).await?;
        self.presence_sub_requests_repo
            .clear_cache(&account)
            .await?;
        Ok(())
    }
}
