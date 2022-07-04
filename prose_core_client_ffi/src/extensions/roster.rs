use crate::error::Result;
use crate::extensions::{XMPPExtension, XMPPExtensionContext};
use crate::helpers::StanzaExt;
use crate::types::namespace::Namespace;
use crate::{types, PresenceKind};
use jid::BareJid;
use libstrophe::Stanza;
use std::sync::{Arc, Mutex};

pub struct Roster {
    ctx: Arc<XMPPExtensionContext>,
    roster: Arc<Mutex<types::roster::Roster>>,
}

impl Roster {
    pub fn new(ctx: Arc<XMPPExtensionContext>) -> Roster {
        Roster {
            ctx,
            roster: Arc::new(Mutex::new(types::roster::Roster::default())),
        }
    }
}

impl XMPPExtension for Roster {
    fn handle_presence_stanza(&self, stanza: &Stanza) -> Result<()> {
        let presence: types::presence::Presence = stanza.try_into()?;
        if presence.kind == Some(PresenceKind::Subscribe) {
            if let Some(from) = presence.from {
                self.ctx
                    .observer
                    .did_receive_presence_subscription_request(from);
            }
        }
        Ok(())
    }

    fn handle_iq_stanza(&self, stanza: &Stanza) -> Result<()> {
        if !stanza.has_namespace(Namespace::Roster) {
            return Ok(());
        }
        let updated_roster: types::roster::Roster = stanza.try_into()?;
        let mut roster = self.roster.lock()?;
        *roster = updated_roster;
        self.ctx.observer.did_receive_roster(roster.clone());
        Ok(())
    }
}

impl Roster {
    pub fn load_roster(&self) -> Result<()> {
        let mut iq_stanza = Stanza::new_iq(Some("get"), Some(&self.ctx.generate_id()));
        iq_stanza.add_child(Stanza::new_query(Namespace::Roster)?)?;
        self.ctx.send_stanza(iq_stanza)
    }

    pub fn add_user(
        &self,
        jid: &BareJid,
        nickname: Option<&str>,
        groups: &[impl AsRef<str>],
    ) -> Result<()> {
        let mut item = Stanza::new();
        item.set_name("item")?;
        item.set_attribute("jid", &jid.to_string())?;

        if let Some(nickname) = nickname {
            item.set_attribute("name", nickname)?;
        }

        for group in groups {
            let mut group_node = Stanza::new();
            group_node.set_name("group")?;

            let mut text_node = Stanza::new();
            text_node.set_text(group)?;
            group_node.add_child(text_node)?;

            item.add_child(group_node)?;
        }

        let mut query = Stanza::new_query(Namespace::Roster)?;
        query.add_child(item)?;

        let mut iq_stanza = Stanza::new_iq(Some("set"), Some(&self.ctx.generate_id()));
        iq_stanza.add_child(query)?;
        self.ctx.send_stanza(iq_stanza)
    }

    pub fn remove_user_and_unsubscribe_from_presence(&self, jid: &BareJid) -> Result<()> {
        let mut item = Stanza::new();
        item.set_name("item")?;
        item.set_attribute("jid", &jid.to_string())?;
        item.set_attribute("subscription", "remove")?;

        let mut query = Stanza::new_query(Namespace::Roster)?;
        query.add_child(item)?;

        let mut iq_stanza = Stanza::new_iq(Some("set"), Some(&self.ctx.generate_id()));
        iq_stanza.add_child(query)?;
        self.ctx.send_stanza(iq_stanza)
    }

    pub fn subscribe_to_user_presence(&self, jid: &BareJid) -> Result<()> {
        let mut presence_stanza = Stanza::new_presence();
        presence_stanza.set_id(self.ctx.generate_id())?;
        presence_stanza.set_to(&jid.to_string())?;
        presence_stanza.set_attribute("type", PresenceKind::Subscribe.to_string())?;
        self.ctx.send_stanza(presence_stanza)
    }

    pub fn unsubscribe_from_user_presence(&self, jid: &BareJid) -> Result<()> {
        let mut presence_stanza = Stanza::new_presence();
        presence_stanza.set_id(self.ctx.generate_id())?;
        presence_stanza.set_to(&jid.to_string())?;
        presence_stanza.set_attribute("type", PresenceKind::Unsubscribe.to_string())?;
        self.ctx.send_stanza(presence_stanza)
    }

    pub fn grant_presence_permission_to_user(&self, jid: &BareJid) -> Result<()> {
        let mut presence_stanza = Stanza::new_presence();
        presence_stanza.set_id(self.ctx.generate_id())?;
        presence_stanza.set_to(&jid.to_string())?;
        presence_stanza.set_attribute("type", PresenceKind::Subscribed.to_string())?;
        self.ctx.send_stanza(presence_stanza)
    }

    pub fn revoke_or_reject_presence_permission_from_user(&self, jid: &BareJid) -> Result<()> {
        let mut presence_stanza = Stanza::new_presence();
        presence_stanza.set_id(self.ctx.generate_id())?;
        presence_stanza.set_to(&jid.to_string())?;
        presence_stanza.set_attribute("type", PresenceKind::Unsubscribed.to_string())?;
        self.ctx.send_stanza(presence_stanza)
    }
}
