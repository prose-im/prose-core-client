use std::sync::Arc;

use jid::{BareJid, FullJid, Jid};
use strum_macros::Display;

use crate::modules::module::Module;
use crate::stanza::message::{chat_marker, ChatMarker, Fallback, Kind, MessageFastening};
use crate::stanza::{iq, message, ForwardedMessage, Message, Namespace, Stanza, StanzaBase, IQ};

use super::super::Context;

#[derive(Debug, Display)]
pub enum ReceivedMessage<'a> {
    Message(&'a Message<'a>),
    ReceivedCarbon(&'a ForwardedMessage<'a>),
    SentCarbon(&'a ForwardedMessage<'a>),
}

impl<'a> ReceivedMessage<'a> {
    pub fn is_carbon(&self) -> bool {
        match self {
            Self::Message(_) => false,
            Self::ReceivedCarbon(_) | Self::SentCarbon(_) => true,
        }
    }
}

pub trait ChatDelegate: Send + Sync {
    fn did_receive_message(&self, message: ReceivedMessage);
    fn will_send_message(&self, message: &Message);
}

pub struct Chat {
    delegate: Option<Arc<dyn ChatDelegate>>,
}

impl Chat {
    pub fn new(delegate: Option<Arc<dyn ChatDelegate + 'static>>) -> Self {
        Chat { delegate }
    }
}

impl Module for Chat {
    fn handle_connect(&self, ctx: &Context) -> anyhow::Result<()> {
        self.set_message_carbons_enabled(ctx, true)
    }

    fn handle_message_stanza(&self, ctx: &Context, stanza: &Message) -> anyhow::Result<()> {
        let Some(delegate) = &self.delegate else {
            return Ok(())
        };

        // Ignore MAM messages.
        if stanza.child_by_name("result").is_some() {
            return Ok(());
        }

        // CVE-2017-5589
        // https://rt-solutions.de/en/cve-2017-5589_xmpp_carbons/
        fn is_valid_carbon(stanza: &Message, account: &FullJid) -> bool {
            let Some(from) = stanza.from() else {
                return false
            };
            from == BareJid::from(account.clone())
        }

        if let Some(received_node) =
            stanza.child_by_name_and_namespace("received", Namespace::MessageCarbons)
        {
            // Ignore messages from invalid senders.
            if !is_valid_carbon(stanza, ctx.jid) {
                return Ok(());
            }
            let Some(message) = received_node.child_by_name_and_namespace("forwarded", Namespace::Forward) else {
                return Ok(())
            };
            delegate.did_receive_message(ReceivedMessage::ReceivedCarbon(&message.into()));
            return Ok(());
        }

        if let Some(sent_node) =
            stanza.child_by_name_and_namespace("sent", Namespace::MessageCarbons)
        {
            // Ignore messages from invalid senders.
            if !is_valid_carbon(stanza, ctx.jid) {
                return Ok(());
            }
            let Some(message) = sent_node.child_by_name_and_namespace("forwarded", Namespace::Forward) else {
                return Ok(())
            };
            delegate.did_receive_message(ReceivedMessage::SentCarbon(&message.into()));
            return Ok(());
        }

        delegate.did_receive_message(ReceivedMessage::Message(stanza));
        Ok(())
    }
}

impl Chat {
    pub fn send_message(
        &self,
        ctx: &Context,
        to: impl Into<Jid>,
        body: impl AsRef<str>,
        chat_state: Option<message::ChatState>,
    ) -> anyhow::Result<()> {
        let mut stanza = Message::new()
            .set_kind(Kind::Chat)
            .set_id(ctx.generate_id().into())
            .set_from(ctx.jid.clone())
            .set_to(to)
            .set_body(body)
            .set_markable();

        if let Some(chat_state) = chat_state {
            stanza = stanza.set_chat_state(chat_state);
        }

        self.send_message_stanza(ctx, stanza)
    }

    pub fn update_message(
        &self,
        ctx: &Context,
        id: message::Id,
        to: impl Into<Jid>,
        body: impl AsRef<str>,
    ) -> anyhow::Result<()> {
        let stanza = Message::new()
            .set_id(ctx.generate_id().into())
            .set_from(ctx.jid.clone())
            .set_to(to)
            .set_body(body)
            .set_replace(id);
        self.send_message_stanza(ctx, stanza)
    }

    pub fn send_chat_state(
        &self,
        ctx: &Context,
        to: impl Into<Jid>,
        chat_state: message::ChatState,
    ) -> anyhow::Result<()> {
        let stanza = Message::new()
            .set_kind(Kind::Chat)
            .set_from(ctx.jid.clone())
            .set_to(to)
            .set_chat_state(chat_state);
        ctx.send_stanza(stanza);
        Ok(())
    }

    // https://xmpp.org/extensions/xep-0444.html
    pub fn react_to_message(
        &self,
        ctx: &Context,
        id: message::Id,
        to: impl Into<Jid>,
        reactions: impl IntoIterator<Item = message::Emoji>,
    ) -> anyhow::Result<()> {
        let stanza = Message::new()
            .set_kind(Kind::Chat)
            .set_id(ctx.generate_id().into())
            .set_from(ctx.jid.clone())
            .set_to(to)
            .set_message_reactions(id, reactions);
        self.send_message_stanza(ctx, stanza)
    }

    // https://xmpp.org/extensions/xep-0424.html
    pub fn retract_message(
        &self,
        ctx: &Context,
        id: message::Id,
        to: impl Into<Jid>,
    ) -> anyhow::Result<()> {
        let stanza = Message::new()
            .set_kind(Kind::Chat)
            .set_id(ctx.generate_id().into())
            .set_from(ctx.jid.clone())
            .set_to(to)
            .set_body("This person attempted to retract a previous message, but it's unsupported by your client.")
            .set_fastening(MessageFastening::new(id, true))
            .set_fallback(Fallback::new(None))
            .add_child(
                Stanza::new("fallback").set_namespace(Namespace::Fallback)
            );
        self.send_message_stanza(ctx, stanza)
    }

    pub fn set_message_carbons_enabled(&self, ctx: &Context, enabled: bool) -> anyhow::Result<()> {
        let stanza = IQ::new(iq::Kind::Set, ctx.generate_id())
            .set_from(ctx.jid.clone())
            .add_child(
                Stanza::new(if enabled { "enable" } else { "disable" })
                    .set_namespace(Namespace::MessageCarbons),
            );
        ctx.send_stanza(stanza);
        Ok(())
    }

    pub fn mark_message(
        &self,
        ctx: &Context,
        id: &message::Id,
        to: impl Into<Jid>,
        marker: chat_marker::Kind,
    ) -> anyhow::Result<()> {
        let stanza = Message::new()
            .set_kind(Kind::Chat)
            .set_id(ctx.generate_id().into())
            .set_from(ctx.jid.clone())
            .set_to(to)
            .add_marker(ChatMarker::new(marker, id));
        self.send_message_stanza(ctx, stanza)
    }
}

impl Chat {
    fn send_message_stanza(&self, ctx: &Context, message: Message) -> anyhow::Result<()> {
        if let Some(delegate) = &self.delegate {
            delegate.will_send_message(&message);
        }
        ctx.send_stanza(message);
        Ok(())
    }
}
