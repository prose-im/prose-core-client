use std::sync::Arc;

use jid::Jid;

use crate::modules::{Context, Module};
use crate::stanza::{pubsub, Message, Namespace, Presence, IQ};

pub(crate) struct CompoundModule {
    modules: Vec<Arc<dyn Module + Send + Sync>>,
}

impl CompoundModule {
    pub fn new() -> Self {
        CompoundModule { modules: vec![] }
    }

    pub fn add_module(&mut self, module: Arc<dyn Module + Send + Sync>) {
        self.modules.push(module)
    }
}

impl Module for CompoundModule {
    fn handle_connect(&self, ctx: &Context) -> anyhow::Result<()> {
        for module in &self.modules {
            module.handle_connect(ctx)?
        }
        Ok(())
    }

    fn handle_disconnect(&self) -> anyhow::Result<()> {
        for module in &self.modules {
            module.handle_disconnect()?
        }
        Ok(())
    }

    fn handle_presence_stanza(&self, ctx: &Context, stanza: &Presence) -> anyhow::Result<()> {
        for module in &self.modules {
            module.handle_presence_stanza(ctx, stanza)?
        }
        Ok(())
    }

    fn handle_message_stanza(&self, ctx: &Context, stanza: &Message) -> anyhow::Result<()> {
        for module in &self.modules {
            module.handle_message_stanza(ctx, stanza)?
        }
        Ok(())
    }

    fn handle_iq_stanza(&self, ctx: &Context, stanza: &IQ) -> anyhow::Result<()> {
        for module in &self.modules {
            module.handle_iq_stanza(ctx, stanza)?
        }
        Ok(())
    }

    fn handle_pubsub_event(
        &self,
        ctx: &Context,
        from: &Jid,
        node: &Namespace,
        event: &pubsub::Event,
    ) -> anyhow::Result<()> {
        for module in &self.modules {
            module.handle_pubsub_event(ctx, from, node, event)?
        }
        Ok(())
    }
}
