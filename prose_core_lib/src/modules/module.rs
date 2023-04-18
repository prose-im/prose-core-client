use jid::Jid;

use crate::stanza::{pubsub, Message, Namespace, Presence, IQ};

use super::Context;

#[allow(unused_variables)]
pub trait Module {
    fn handle_connect(&self, ctx: &Context) -> anyhow::Result<()> {
        Ok(())
    }
    fn handle_disconnect(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn handle_presence_stanza(&self, ctx: &Context, stanza: &Presence) -> anyhow::Result<()> {
        Ok(())
    }
    fn handle_message_stanza(&self, ctx: &Context, stanza: &Message) -> anyhow::Result<()> {
        Ok(())
    }
    fn handle_iq_stanza(&self, ctx: &Context, stanza: &IQ) -> anyhow::Result<()> {
        Ok(())
    }
    fn handle_pubsub_event(
        &self,
        ctx: &Context,
        from: &Jid,
        node: &Namespace,
        event: &pubsub::Event,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
