// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{format_err, Result};
use jid::BareJid;
use tracing::error;
use xmpp_parsers::iq::{Iq, IqType};
use xmpp_parsers::nick::Nick;
use xmpp_parsers::presence::{Presence, Type};
use xmpp_parsers::roster::{Group, Item, Roster as Query, Subscription};

use crate::client::ModuleContext;
use crate::event::Event as ClientEvent;
use crate::mods::roster::Event::{PresenceSubscriptionRequest, RosterItemChanged};
use crate::mods::Module;
use crate::ns;
use crate::util::RequestError;

#[derive(Default, Clone)]
pub struct Roster {
    ctx: ModuleContext,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    PresenceSubscriptionRequest {
        from: BareJid,
        nickname: Option<String>,
    },
    RosterItemChanged {
        item: Item,
    },
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

        let nickname = stanza
            .payloads
            .iter()
            .find(|p| p.is("nick", ns::NICK))
            .cloned()
            .and_then(|p| Nick::try_from(p).ok())
            .map(|nick| nick.0);

        self.ctx
            .schedule_event(ClientEvent::Roster(PresenceSubscriptionRequest {
                from: jid.to_bare(),
                nickname,
            }));

        Ok(())
    }

    fn handle_iq_stanza(&self, stanza: &Iq) -> Result<()> {
        // A "roster push" is a newly created, updated, or deleted roster item that is sent from
        // the server to the client; syntactically it is an IQ stanza of type "set" sent from
        // server to client and containing a <query/> element qualified by
        // the 'jabber:iq:roster' namespace.

        let IqType::Set(payload) = &stanza.payload else {
            return Ok(());
        };

        if !payload.is("query", ns::ROSTER) {
            return Ok(());
        }

        let mut query = Query::try_from(payload.clone())?;

        // The following rules apply to roster pushes:
        // 1. The <query/> element in a roster push MUST contain one and only one <item/> element.
        // 2. A receiving client MUST ignore the stanza unless it has no 'from' attribute
        //    (i.e., implicitly from the bare JID of the user's account) or it has a 'from'
        //    attribute whose value matches the user's bare JID <user@domainpart>.

        let item = query.items.pop().ok_or(format_err!(
            "Encountered invalid roster push. Query element did not contain an item."
        ))?;

        match &stanza.from {
            None => (),
            Some(jid) if jid.to_bare() == self.ctx.bare_jid() => (),
            Some(jid) => {
                error!("Received roster push from invalid sender {}.", jid)
            }
        }

        self.handle_roster_push(stanza, item)?;
        Ok(())
    }
}

impl Roster {
    /// https://xmpp.org/rfcs/rfc6121.html#roster-syntax-actions-push
    fn handle_roster_push(&self, iq: &Iq, item: Item) -> Result<()> {
        self.ctx
            .schedule_event(ClientEvent::Roster(RosterItemChanged { item }));

        // As mandated by the semantics of the IQ stanza as defined in [XMPP‑CORE],
        // each resource that receives a roster push from the server is supposed to reply with an
        // IQ stanza of type "result" or "error" (however, it is known that many existing clients
        // do not reply to roster pushes).

        self.ctx.send_stanza(Iq {
            from: None,
            to: iq.from.clone(),
            id: iq.id.clone(),
            payload: IqType::Result(None),
        })?;

        Ok(())
    }
}

impl Roster {
    /// https://xmpp.org/rfcs/rfc6121.html#roster-login
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

    /// https://xmpp.org/rfcs/rfc6121.html#roster-add
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

    /// https://xmpp.org/rfcs/rfc6121.html#roster-delete
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

    /// https://xmpp.org/rfcs/rfc6121.html#sub-preapproval
    pub async fn preapprove_subscription_request(&self, jid: &BareJid) -> Result<()> {
        self.ctx
            .send_stanza(Presence::new(Type::Subscribed).with_to(jid.clone()))?;
        Ok(())
    }

    /// https://xmpp.org/rfcs/rfc6121.html#sub-request
    pub async fn subscribe_to_presence(&self, jid: &BareJid) -> Result<()> {
        self.ctx
            .send_stanza(Presence::new(Type::Subscribe).with_to(jid.clone()))?;
        Ok(())
    }

    /// https://xmpp.org/rfcs/rfc6121.html#sub-unsub
    pub async fn unsubscribe_from_presence(&self, jid: &BareJid) -> Result<()> {
        self.ctx
            .send_stanza(Presence::new(Type::Unsubscribe).with_to(jid.clone()))?;
        Ok(())
    }

    /// https://xmpp.org/rfcs/rfc6121.html#sub-cancel
    pub async fn revoke_presence_subscription(&self, jid: &BareJid) -> Result<()> {
        self.ctx
            .send_stanza(Presence::new(Type::Unsubscribed).with_to(jid.clone()))?;
        Ok(())
    }

    /// https://xmpp.org/rfcs/rfc6121.html#sub-request-handle
    pub async fn approve_presence_subscription_request(&self, from: &BareJid) -> Result<()> {
        self.ctx
            .send_stanza(Presence::new(Type::Subscribed).with_to(from.clone()))?;
        Ok(())
    }

    /// https://xmpp.org/rfcs/rfc6121.html#sub-request-handle
    pub async fn deny_presence_subscription_request(&self, from: &BareJid) -> Result<()> {
        self.ctx
            .send_stanza(Presence::new(Type::Unsubscribed).with_to(from.clone()))?;
        Ok(())
    }
}
