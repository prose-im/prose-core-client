// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{bail, Result};
use jid::Jid;
use xmpp_parsers::disco::{DiscoInfoQuery, DiscoInfoResult, DiscoItemsQuery, DiscoItemsResult};
use xmpp_parsers::iq::{Iq, IqType};
use xmpp_parsers::presence::Presence;
use xmpp_parsers::{disco, ns, presence};

use crate::client::ModuleContext;
use crate::event::Event as ClientEvent;
use crate::mods::Module;
use crate::util::RequestError;

/// XEP-0115: Entity Capabilities
/// https://xmpp.org/extensions/xep-0115.html
#[derive(Default, Clone)]
pub struct Caps {
    ctx: ModuleContext,
}

#[derive(Debug, Clone)]
pub enum Event {
    DiscoInfoQuery {
        from: Jid,
        id: String,
        node: Option<String>,
    },
    Caps {
        from: Jid,
        caps: xmpp_parsers::caps::Caps,
    },
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Event::DiscoInfoQuery {
                    from: f1,
                    id: i1,
                    node: n1,
                },
                Event::DiscoInfoQuery {
                    from: f2,
                    id: i2,
                    node: n2,
                },
            ) => f1 == f2 && i1 == i2 && n1 == n2,
            (Event::Caps { from: f1, caps: c1 }, Event::Caps { from: f2, caps: c2 }) => {
                f1 == f2 && c1.ext == c2.ext && c1.node == c2.node && c1.hash == c2.hash
            }
            (Event::DiscoInfoQuery { .. }, _) => false,
            (Event::Caps { .. }, _) => false,
        }
    }
}

impl Module for Caps {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context
    }

    fn handle_presence_stanza(&self, stanza: &Presence) -> Result<()> {
        let (Some(from), Some(caps)) = (
            &stanza.from,
            stanza.payloads.iter().find(|p| p.is("c", ns::CAPS)),
        ) else {
            return Ok(());
        };

        self.ctx.schedule_event(ClientEvent::Caps(Event::Caps {
            from: from.clone(),
            caps: xmpp_parsers::caps::Caps::try_from(caps.clone())?,
        }));
        Ok(())
    }

    fn handle_iq_stanza(&self, stanza: &Iq) -> Result<()> {
        let IqType::Get(payload) = &stanza.payload else {
            return Ok(());
        };

        if !payload.is("query", ns::DISCO_INFO) {
            return Ok(());
        }

        let query = DiscoInfoQuery::try_from(payload.clone())?;

        let Some(from) = &stanza.from else {
            bail!("Missing 'from' in disco request.")
        };

        self.ctx
            .schedule_event(ClientEvent::Caps(Event::DiscoInfoQuery {
                from: from.clone(),
                id: stanza.id.clone(),
                node: query.node,
            }));

        Ok(())
    }
}

impl Caps {
    pub fn publish_capabilities(&self, caps: xmpp_parsers::caps::Caps) -> Result<()> {
        let mut presence = Presence::new(presence::Type::None);
        presence.add_payload(caps);
        self.ctx.send_stanza(presence)?;
        Ok(())
    }

    pub async fn send_disco_info_query_response(
        &self,
        to: impl Into<Jid>,
        id: String,
        disco: disco::DiscoInfoResult,
    ) -> Result<()> {
        self.ctx
            .send_stanza(Iq::from_result(id, Some(disco)).with_to(to.into()))?;
        Ok(())
    }

    pub async fn query_server_disco_info(
        &self,
        node: Option<String>,
    ) -> Result<DiscoInfoResult, RequestError> {
        self.query_disco_info(self.ctx.server_jid(), node).await
    }

    pub async fn query_server_disco_items(
        &self,
        node: Option<String>,
    ) -> Result<DiscoItemsResult, RequestError> {
        self.query_disco_items(self.ctx.server_jid(), node).await
    }

    pub async fn query_disco_items(
        &self,
        from: impl Into<Jid>,
        node: Option<String>,
    ) -> Result<DiscoItemsResult, RequestError> {
        let response = self
            .ctx
            .send_iq(
                Iq::from_get(self.ctx.generate_id(), DiscoItemsQuery { node, rsm: None })
                    .with_to(from.into()),
            )
            .await?
            .ok_or(RequestError::UnexpectedResponse)?;

        Ok(DiscoItemsResult::try_from(response)?)
    }

    pub async fn query_disco_info(
        &self,
        from: impl Into<Jid>,
        node: Option<String>,
    ) -> Result<DiscoInfoResult, RequestError> {
        let response = self
            .ctx
            .send_iq(
                Iq::from_get(self.ctx.generate_id(), DiscoInfoQuery { node }).with_to(from.into()),
            )
            .await?
            .ok_or(RequestError::UnexpectedResponse)?;

        Ok(DiscoInfoResult::try_from(response)?)
    }
}
