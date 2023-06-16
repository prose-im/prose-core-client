use std::sync::Arc;

use jid::Jid;

use crate::modules::{Context, Module, XMPPElement};
use crate::stanza::{pubsub, Message, Namespace, Presence, Stanza, IQ};

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

    fn handle_element(&self, ctx: &Context, element: &XMPPElement) -> anyhow::Result<()> {
        for module in &self.modules {
            module.handle_element(ctx, element)?
        }
        Ok(())
    }
}
