// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use xmpp_parsers::roster::{Group, Item};

use prose_xmpp::mods;

use crate::domain::contacts::models::{Contact, Group as DomainGroup};
use crate::domain::contacts::services::ContactsService;
use crate::infra::xmpp::XMPPClient;

#[async_trait]
impl ContactsService for XMPPClient {
    async fn load_contacts(&self, account_jid: &BareJid) -> Result<Vec<Contact>> {
        let roster = self.client.get_mod::<mods::Roster>();

        let contacts = roster
            .load_roster()
            .await?
            .items
            .into_iter()
            .map(|item| Contact::from((account_jid, item)))
            .collect::<Vec<_>>();

        Ok(contacts)
    }

    async fn add_contact(&self, jid: &BareJid) -> Result<()> {
        let roster_mod = self.client.get_mod::<mods::Roster>();
        roster_mod.add_contact(jid, None, None).await?;
        roster_mod.subscribe_to_presence(jid).await?;
        Ok(())
    }
}

impl From<(&BareJid, Item)> for Contact {
    fn from((account_jid, roster_item): (&BareJid, Item)) -> Self {
        let favorites = Group("Favorite".to_string());

        let group = match roster_item.groups {
            _ if roster_item.groups.contains(&favorites) => DomainGroup::Favorite,
            _ if roster_item.jid.domain() == account_jid.domain() => DomainGroup::Team,
            _ => DomainGroup::Other,
        };

        Contact {
            jid: roster_item.jid,
            name: roster_item.name,
            group,
        }
    }
}
