use jid::Jid;

use crate::stanza::{pubsub, Message, Namespace, Presence, Stanza, IQ};

use super::Context;

#[derive(Debug)]
pub enum XMPPElement<'a> {
    Presence(Presence<'a>),
    Message(Message<'a>),
    IQ(IQ<'a>),
    PubSubEvent {
        from: Jid,
        node: Namespace,
        event: pubsub::Event<'a>,
    },
    Other(Stanza<'a>),
}

#[allow(unused_variables)]
pub trait Module {
    fn handle_connect(&self, ctx: &Context) -> anyhow::Result<()> {
        Ok(())
    }
    fn handle_disconnect(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn handle_element(&self, ctx: &Context, element: &XMPPElement) -> anyhow::Result<()> {
        match element {
            XMPPElement::Presence(ref p) => self.handle_presence_stanza(ctx, p),
            XMPPElement::Message(ref m) => self.handle_message_stanza(ctx, m),
            XMPPElement::IQ(ref i) => self.handle_iq_stanza(ctx, i),
            XMPPElement::PubSubEvent {
                ref from,
                ref node,
                ref event,
            } => self.handle_pubsub_event(ctx, from, node, event),
            XMPPElement::Other(ref o) => Ok(()),
        }
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
