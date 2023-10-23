// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::BareJid;
use minidom::Element;
use xmpp_parsers::iq::{Iq, IqGetPayload, IqSetPayload};
use xmpp_parsers::presence::{Presence, Type};

use crate::client::ModuleContext;
use crate::event::Event as ClientEvent;
use crate::mods::roster::Event::PresenceSubscriptionRequest;
use crate::mods::Module;
use crate::ns;
use crate::util::{ElementExt, RequestError};

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
                Query::new(self.ctx.generate_id()),
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
                query_id: self.ctx.generate_id(),
                item: Some(Item {
                    jid: jid.clone(),
                    name: name.map(ToString::to_string),
                    group: group.map(ToString::to_string),
                }),
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

struct Query {
    query_id: String,
    item: Option<Item>,
}

impl Query {
    fn new(query_id: impl Into<String>) -> Self {
        Query {
            query_id: query_id.into(),
            item: None,
        }
    }
}

impl From<Query> for Element {
    fn from(value: Query) -> Self {
        Element::builder("query", ns::ROSTER)
            .attr("queryid", value.query_id)
            .append_all(value.item)
            .build()
    }
}

impl TryFrom<Element> for Query {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        Ok(Query {
            query_id: value.attr_req("queryid")?.to_string(),
            item: None,
        })
    }
}

impl IqGetPayload for Query {}
impl IqSetPayload for Query {}

struct Item {
    jid: BareJid,
    name: Option<String>,
    group: Option<String>,
}

impl From<Item> for Element {
    fn from(value: Item) -> Self {
        Element::builder("item", ns::ROSTER)
            .attr("jid", value.jid)
            .attr("name", value.name)
            .append_all(
                value
                    .group
                    .map(|group| Element::builder("group", ns::ROSTER).append(group)),
            )
            .build()
    }
}
