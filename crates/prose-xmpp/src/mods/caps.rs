use anyhow::Result;
use jid::Jid;
use xmpp_parsers::disco::DiscoInfoQuery;
use xmpp_parsers::iq::{Iq, IqType};
use xmpp_parsers::presence::Presence;
use xmpp_parsers::{disco, ns, presence};

use crate::client::ModuleContext;
use crate::event::Event;
use crate::mods::Module;

#[derive(Default, Clone)]
pub struct Caps {
    ctx: ModuleContext,
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

        self.ctx.schedule_event(Event::CapsPresence {
            from: from.clone(),
            caps: xmpp_parsers::caps::Caps::try_from(caps.clone())?,
        });
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

        let (Some(node), Some(from)) = (query.node, &stanza.from) else {
            return Ok(());
        };

        self.ctx.schedule_event(Event::DiscoInfoQuery {
            from: from.clone(),
            id: stanza.id.clone(),
            node,
        });

        Ok(())
    }
}

impl Caps {
    pub fn publish_capabilities(&self, caps: xmpp_parsers::caps::Caps) -> Result<()> {
        let mut presence = Presence::new(presence::Type::None);
        presence.add_payload(caps);
        self.ctx.send_stanza(presence)
    }

    pub async fn send_disco_info_query_response(
        &self,
        to: impl Into<Jid>,
        id: String,
        disco: disco::DiscoInfoResult,
    ) -> Result<()> {
        self.ctx
            .send_stanza(Iq::from_result(id, Some(disco)).with_to(to.into()))
    }

    pub async fn query_server_features(&self) -> Result<()> {
        let stanza = self
            .ctx
            .send_iq(Iq::from_get(
                self.ctx.generate_id(),
                DiscoInfoQuery { node: None },
            ))
            .await?;

        if let Some(stanza) = stanza {
            println!("{}", String::from(&stanza));
        }
        Ok(())
    }
}
