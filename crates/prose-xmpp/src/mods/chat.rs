// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::Jid;
use xmpp_parsers::carbons;
use xmpp_parsers::iq::Iq;

use crate::client::ModuleContext;
use crate::event::Event as ClientEvent;
use crate::mods::Module;
use crate::stanza::message;
use crate::stanza::message::chat_marker::Received;
use crate::stanza::message::fasten::ApplyTo;
use crate::stanza::message::retract::Retract;
use crate::stanza::message::{
    ChatState, Emoji, Fallback, Forwarded, Message, MessageType, Reactions,
};

#[derive(Debug, Clone)]
pub enum Carbon {
    Received(Forwarded),
    Sent(Forwarded),
}

#[derive(Default, Clone)]
pub struct Chat {
    ctx: ModuleContext,
}

#[derive(Debug, Clone)]
pub enum Event {
    Message(Message),
    Carbon(Carbon),
    Sent(Message),
}

impl Module for Chat {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context
    }

    fn handle_message_stanza(&self, stanza: &Message) -> Result<()> {
        // Ignore MAM messages.
        if stanza.archived_message.is_some() {
            return Ok(());
        }

        if let Some(received_carbon) = &stanza.received_carbon {
            // Ignore messages from invalid senders.
            // CVE-2017-5589
            // https://rt-solutions.de/en/cve-2017-5589_xmpp_carbons/
            if stanza.from == Some(Jid::Bare(self.ctx.bare_jid())) {
                self.ctx
                    .schedule_event(ClientEvent::Chat(Event::Carbon(Carbon::Received(
                        received_carbon.forwarded.clone(),
                    ))));
            }
            return Ok(());
        }

        if let Some(sent_carbon) = &stanza.sent_carbon {
            // Ignore messages from invalid senders.
            // CVE-2017-5589
            // https://rt-solutions.de/en/cve-2017-5589_xmpp_carbons/
            if stanza.from == Some(self.ctx.bare_jid().into()) {
                self.ctx
                    .schedule_event(ClientEvent::Chat(Event::Carbon(Carbon::Sent(
                        sent_carbon.forwarded.clone(),
                    ))));
            }
            return Ok(());
        }

        self.ctx
            .schedule_event(ClientEvent::Chat(Event::Message(stanza.clone())));

        Ok(())
    }
}

impl Chat {
    pub fn send_message(
        &self,
        to: impl Into<Jid>,
        body: impl Into<String>,
        chat_state: Option<ChatState>,
    ) -> Result<()> {
        let mut stanza = Message::new()
            .set_type(MessageType::Chat)
            .set_id(self.ctx.generate_id().into())
            .set_from(self.ctx.full_jid())
            .set_to(to)
            .set_body(body)
            .set_markable();

        stanza.chat_state = chat_state;

        self.send_message_stanza(stanza)
    }

    pub fn update_message(
        &self,
        id: message::Id,
        to: impl Into<Jid>,
        body: impl Into<String>,
    ) -> Result<()> {
        let stanza = Message::new()
            .set_id(self.ctx.generate_id().into())
            .set_from(self.ctx.full_jid())
            .set_to(to)
            .set_body(body)
            .set_replace(id);
        self.send_message_stanza(stanza)
    }

    pub fn send_chat_state(&self, to: impl Into<Jid>, chat_state: ChatState) -> Result<()> {
        let stanza = Message::new()
            .set_type(MessageType::Chat)
            .set_from(self.ctx.full_jid())
            .set_to(to)
            .set_chat_state(chat_state);
        self.ctx.send_stanza(stanza)
    }

    // https://xmpp.org/extensions/xep-0444.html
    pub fn react_to_message(
        &self,
        id: message::Id,
        to: impl Into<Jid>,
        reactions: impl IntoIterator<Item = Emoji>,
    ) -> Result<()> {
        let stanza = Message::new()
            .set_type(MessageType::Chat)
            .set_id(self.ctx.generate_id().into())
            .set_from(self.ctx.full_jid())
            .set_to(to)
            .set_store(true)
            .set_message_reactions(Reactions {
                id,
                reactions: reactions.into_iter().collect(),
            });
        self.send_message_stanza(stanza)
    }

    // https://xmpp.org/extensions/xep-0424.html
    pub fn retract_message(&self, id: message::Id, to: impl Into<Jid>) -> Result<()> {
        let stanza = Message::new()
        .set_type(MessageType::Chat)
        .set_id(self.ctx.generate_id().into())
        .set_from(self.ctx.full_jid())
        .set_to(to)
        .set_body("This person attempted to retract a previous message, but it's unsupported by your client.")
        .set_fastening(ApplyTo::new(id).with_payload(Retract::default()))
        .set_fallback(Fallback {
          r#for: None,
          subjects: vec![],
          bodies: vec![],
        });
        self.send_message_stanza(stanza)
    }

    pub fn set_message_carbons_enabled(&self, enabled: bool) -> Result<()> {
        if enabled {
            self.ctx
                .send_stanza(Iq::from_set(self.ctx.generate_id(), carbons::Enable))
        } else {
            self.ctx
                .send_stanza(Iq::from_set(self.ctx.generate_id(), carbons::Disable))
        }
    }

    pub fn mark_message_received(&self, id: message::Id, to: impl Into<Jid>) -> Result<()> {
        let stanza = Message::new()
            .set_type(MessageType::Chat)
            .set_id(self.ctx.generate_id().into())
            .set_from(self.ctx.full_jid().clone())
            .set_to(to)
            .set_received_marker(Received { id });
        self.send_message_stanza(stanza)
    }
}

impl Chat {
    fn send_message_stanza(&self, message: Message) -> Result<()> {
        self.ctx
            .schedule_event(ClientEvent::Chat(Event::Sent(message.clone())));
        self.ctx.send_stanza(message)
    }
}
