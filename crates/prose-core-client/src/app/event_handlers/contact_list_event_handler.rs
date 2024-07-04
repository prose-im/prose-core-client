// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{DynContactListDomainService, DynUserInfoDomainService};
use crate::app::event_handlers::{
    ContactListEvent, ContactListEventType, ServerEvent, ServerEventHandler,
};
use crate::domain::contacts::models::Contact;
use crate::dtos::PresenceSubscription;

/// Handles contact list related events.
#[derive(InjectDependencies)]
pub struct ContactListEventHandler {
    #[inject]
    contact_list_domain_service: DynContactListDomainService,
    #[inject]
    user_info_domain_service: DynUserInfoDomainService,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ServerEventHandler for ContactListEventHandler {
    fn name(&self) -> &'static str {
        "contact_list"
    }

    async fn handle_event(&self, event: ServerEvent) -> Result<Option<ServerEvent>> {
        match event {
            ServerEvent::ContactList(event) => {
                self.handle_contact_list_event(event).await?;
            }
            _ => return Ok(Some(event)),
        }
        Ok(None)
    }
}

impl ContactListEventHandler {
    async fn handle_contact_list_event(&self, event: ContactListEvent) -> Result<()> {
        match event.r#type {
            ContactListEventType::ContactRemoved => {
                self.contact_list_domain_service
                    .handle_removed_contact(&event.contact_id)
                    .await?;
            }
            ContactListEventType::ContactAddedOrPresenceSubscriptionUpdated { subscription } => {
                self.contact_list_domain_service
                    .handle_updated_contact(&event.contact_id, subscription)
                    .await?;
            }
            ContactListEventType::PresenceSubscriptionRequested { nickname } => {
                self.contact_list_domain_service
                    .handle_presence_sub_request(&event.contact_id, nickname.clone())
                    .await?;
                self.user_info_domain_service
                    .handle_contacts_changed(vec![Contact {
                        id: event.contact_id,
                        name: nickname,
                        presence_subscription: PresenceSubscription::Requested,
                    }])
                    .await?;
            }
        }

        Ok(())
    }
}
