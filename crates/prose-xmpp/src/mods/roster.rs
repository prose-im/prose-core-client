// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::BareJid;
use xmpp_parsers::iq::Iq;
use xmpp_parsers::presence::{Presence, Type};
use xmpp_parsers::roster::{Group, Item, Roster as Query, Subscription};

use crate::client::ModuleContext;
use crate::event::Event as ClientEvent;
use crate::mods::roster::Event::PresenceSubscriptionRequest;
use crate::mods::Module;
use crate::util::RequestError;

#[derive(Default, Clone)]
pub struct Roster {
    ctx: ModuleContext,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    PresenceSubscriptionRequest { from: BareJid },
}

impl Module for Roster {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context
    }

    fn handle_presence_stanza(&self, stanza: &Presence) -> Result<()> {
        if stanza.type_ != Type::Subscribe {
            return Ok(());
        }

        let Some(jid) = &stanza.from else {
            return Ok(());
        };

        self.ctx
            .schedule_event(ClientEvent::Roster(PresenceSubscriptionRequest {
                from: jid.to_bare(),
            }));

        Ok(())
    }
}

impl Roster {
    pub async fn load_roster(&self) -> Result<xmpp_parsers::roster::Roster> {
        let roster = self
            .ctx
            .send_iq(Iq::from_get(
                self.ctx.generate_id(),
                Query {
                    ver: None,
                    items: vec![],
                },
            ))
            .await?;

        let Some(response) = roster else {
            return Err(RequestError::UnexpectedResponse.into());
        };

        Ok(xmpp_parsers::roster::Roster::try_from(response)?)
    }

    pub async fn add_contact(
        &self,
        jid: &BareJid,
        name: Option<&str>,
        group: Option<&str>,
    ) -> Result<()> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            Query {
                ver: None,
                items: vec![Item {
                    jid: jid.clone(),
                    name: name.map(ToString::to_string),
                    subscription: Default::default(),
                    ask: Default::default(),
                    groups: group
                        .map(|group| vec![Group(group.to_string())])
                        .unwrap_or_else(|| vec![]),
                }],
            },
        );
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn remove_contact(&self, jid: &BareJid) -> Result<()> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            Query {
                ver: None,
                items: vec![Item {
                    jid: jid.clone(),
                    name: None,
                    subscription: Subscription::Remove,
                    ask: Default::default(),
                    groups: vec![],
                }],
            },
        );
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub async fn subscribe_to_presence(&self, jid: &BareJid) -> Result<()> {
        self.ctx
            .send_stanza(Presence::new(Type::Subscribe).with_to(jid.clone()))?;
        Ok(())
    }

    pub async fn approve_presence_subscription_request(&self, from: &BareJid) -> Result<()> {
        self.ctx
            .send_stanza(Presence::new(Type::Subscribed).with_to(from.clone()))?;
        Ok(())
    }

    pub async fn deny_presence_subscription_request(&self, from: &BareJid) -> Result<()> {
        self.ctx
            .send_stanza(Presence::new(Type::Unsubscribed).with_to(from.clone()))?;
        Ok(())
    }
}
