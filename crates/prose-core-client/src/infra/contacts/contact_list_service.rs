// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_xmpp::mods;

use crate::domain::contacts::models::Contact;
use crate::domain::contacts::services::ContactListService;
use crate::dtos::UserId;
use crate::infra::xmpp::XMPPClient;
use crate::util::jid_workspace::ProseWorkspaceJid;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ContactListService for XMPPClient {
    async fn load_contacts(&self) -> Result<Vec<Contact>> {
        let roster = self.client.get_mod::<mods::Roster>();
        let contacts = roster
            .load_roster()
            .await?
            .items
            .into_iter()
            .filter_map(|item| {
                if item.jid.is_prose_workspace() {
                    return None;
                };
                return Some(Contact::from(item));
            })
            .collect::<Vec<_>>();

        Ok(contacts)
    }

    async fn add_contact(&self, user_id: &UserId) -> Result<()> {
        let roster_mod = self.client.get_mod::<mods::Roster>();
        roster_mod.add_contact(user_id.as_ref(), None, None).await?;
        Ok(())
    }

    async fn remove_contact(&self, user_id: &UserId) -> Result<()> {
        let roster_mod = self.client.get_mod::<mods::Roster>();
        roster_mod.remove_contact(user_id.as_ref()).await?;
        Ok(())
    }

    async fn subscribe_to_presence(&self, user_id: &UserId) -> Result<()> {
        let roster_mod = self.client.get_mod::<mods::Roster>();
        roster_mod.subscribe_to_presence(user_id.as_ref()).await?;
        Ok(())
    }

    async fn unsubscribe_from_presence(&self, user_id: &UserId) -> Result<()> {
        let roster_mod = self.client.get_mod::<mods::Roster>();
        roster_mod
            .unsubscribe_from_presence(user_id.as_ref())
            .await?;
        Ok(())
    }

    async fn revoke_presence_subscription(&self, user_id: &UserId) -> Result<()> {
        let roster_mod = self.client.get_mod::<mods::Roster>();
        roster_mod
            .revoke_presence_subscription(user_id.as_ref())
            .await?;
        Ok(())
    }

    async fn preapprove_subscription_request(&self, user_id: &UserId) -> Result<()> {
        let roster_mod = self.client.get_mod::<mods::Roster>();
        roster_mod
            .preapprove_subscription_request(user_id.as_ref())
            .await?;
        Ok(())
    }

    async fn approve_presence_sub_request(&self, to: &UserId) -> Result<()> {
        let roster_mod = self.client.get_mod::<mods::Roster>();
        roster_mod
            .approve_presence_subscription_request(to.as_ref())
            .await?;
        Ok(())
    }

    async fn deny_presence_sub_request(&self, to: &UserId) -> Result<()> {
        let roster_mod = self.client.get_mod::<mods::Roster>();
        roster_mod
            .deny_presence_subscription_request(to.as_ref())
            .await?;
        Ok(())
    }
}
