use std::sync::Arc;

use jid::Jid;

use crate::modules::caps::DiscoveryInfo;
use crate::modules::{Context, Module};
use crate::stanza;
use crate::stanza::iq::Kind;
use crate::stanza::{Namespace, Presence, Stanza, StanzaBase, IQ};

pub trait CapsDelegate: Send + Sync {
    fn handle_disco_request(&self, node: &str) -> anyhow::Result<DiscoveryInfo<'static>>;
    fn handle_caps_presence(&self, from: &Jid, caps: stanza::presence::Caps);
}

pub struct Caps {
    delegate: Option<Arc<dyn CapsDelegate>>,
}

impl Caps {
    pub fn new(delegate: Option<Arc<dyn CapsDelegate + 'static>>) -> Self {
        Caps { delegate }
    }
}

impl Module for Caps {
    fn handle_iq_stanza(&self, ctx: &Context, stanza: &IQ) -> anyhow::Result<()> {
        let Some(handler) = &self.delegate else {
            return Ok(())
        };

        let (Some(id), Some(from)) = (stanza.id(), stanza.from()) else {
            return Ok(())
        };
        let Some(query) = stanza.child_by_name_and_namespace("query", Namespace::DiscoInfo) else {
            return Ok(())
        };
        let Some(node) = query.attribute("node") else {
            return Ok(())
        };

        let disco = handler.handle_disco_request(node)?;
        ctx.send_stanza(IQ::new(Kind::Result, id).set_to(from).add_child(disco));
        Ok(())
    }

    fn handle_presence_stanza(&self, _ctx: &Context, stanza: &Presence) -> anyhow::Result<()> {
        let Some(handler) = &self.delegate else {
            return Ok(())
        };
        let (Some(from), Some(caps)) = (stanza.from(), stanza.child_by_name_and_namespace("c", Namespace::Caps)) else {
            return Ok(())
        };
        handler.handle_caps_presence(&from, caps.into());
        Ok(())
    }
}

impl Caps {
    pub fn publish_capabilities(
        &self,
        ctx: &Context<'_>,
        caps: stanza::presence::Caps,
    ) -> anyhow::Result<()> {
        ctx.send_stanza(Presence::new().set_caps(caps));
        Ok(())
    }

    pub async fn query_server_features(&self, ctx: &Context<'_>) -> anyhow::Result<()> {
        let stanza = ctx
            .send_iq(
                IQ::new(Kind::Get, ctx.generate_id())
                    .add_child(Stanza::new_query(Namespace::DiscoInfo, None)),
            )
            .await?;
        println!("{}", stanza);
        Ok(())
    }
}
