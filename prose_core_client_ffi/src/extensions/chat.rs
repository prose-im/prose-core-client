use crate::error::Result;
use crate::extensions::{XMPPExtension, XMPPExtensionContext};
use crate::helpers::StanzaExt;
use crate::types::forwarded_message::ForwardedMessage;
use crate::types::message::{ChatState, Message, MessageId};
use crate::types::namespace::Namespace;
use jid::BareJid;
use libstrophe::Stanza;
use std::ops::Deref;
use std::sync::Arc;

pub struct Chat {
    ctx: Arc<XMPPExtensionContext>,
}

impl Chat {
    pub fn new(ctx: Arc<XMPPExtensionContext>) -> Self {
        Chat { ctx }
    }
}

impl XMPPExtension for Chat {
    fn handle_connect(&self) -> Result<()> {
        // Enable message carbons on startup.
        self.set_message_carbons_enabled(true)
    }

    fn handle_message_stanza(&self, stanza: &Stanza) -> Result<()> {
        // Ignore MAM messages.
        if stanza.get_child_by_name("result").is_some() {
            return Ok(());
        }

        if let Some(received_node) =
            stanza.get_child_by_name_and_ns("received", Namespace::MessageCarbons)
        {
            // Ignore messages from invalid senders.
            if stanza.from() != Some(&BareJid::from(self.ctx.jid.clone()).to_string()) {
                return Ok(());
            }

            let forward_node =
                received_node.get_required_child_by_name_and_ns("forwarded", Namespace::Forward)?;
            let message: ForwardedMessage = forward_node.deref().try_into()?;
            self.ctx.observer.did_receive_message_carbon(message);
            return Ok(());
        }

        if let Some(sent_node) = stanza.get_child_by_name_and_ns("sent", Namespace::MessageCarbons)
        {
            // Ignore messages from invalid senders.
            if stanza.from() != Some(&BareJid::from(self.ctx.jid.clone()).to_string()) {
                return Ok(());
            }

            let forward_node =
                sent_node.get_required_child_by_name_and_ns("forwarded", Namespace::Forward)?;
            let message: ForwardedMessage = forward_node.deref().try_into()?;
            self.ctx.observer.did_receive_sent_message_carbon(message);
            return Ok(());
        }

        let message: Message = stanza.try_into()?;
        self.ctx.observer.did_receive_message(message);
        Ok(())
    }
}

impl Chat {
    pub fn send_message(
        &self,
        id: MessageId,
        to: &BareJid,
        body: &str,
        chat_state: Option<ChatState>,
    ) -> Result<()> {
        let mut stanza =
            Stanza::new_message(Some("chat"), Some(id.as_ref()), Some(&to.to_string()));
        stanza.set_body(&body.to_string())?;

        if let Some(chat_state) = chat_state {
            let mut chat_state_node = Stanza::new();
            chat_state_node.set_name(chat_state.to_string())?;
            chat_state_node.set_ns(Namespace::ChatStates)?;
            stanza.add_child(chat_state_node)?;
        }

        self.ctx.send_stanza(stanza)
    }

    pub fn update_message(
        &self,
        id: MessageId,
        new_id: MessageId,
        to: &BareJid,
        body: &str,
    ) -> Result<()> {
        let mut stanza = Stanza::new_message(None, Some(new_id.as_ref()), Some(&to.to_string()));
        stanza.set_body(&body.to_string())?;

        let mut replace_node = Stanza::new();
        replace_node.set_name("replace")?;
        replace_node.set_id(id)?;
        replace_node.set_ns(Namespace::LastMessageCorrection)?;
        stanza.add_child(replace_node)?;

        self.ctx.send_stanza(stanza)
    }

    pub fn send_chat_state(&self, to: &BareJid, chat_state: ChatState) -> Result<()> {
        let mut stanza = Stanza::new_message(Some("chat"), None, Some(&to.to_string()));

        let mut chat_state_node = Stanza::new();
        chat_state_node.set_name(chat_state.to_string())?;
        chat_state_node.set_ns(Namespace::ChatStates)?;
        stanza.add_child(chat_state_node)?;

        self.ctx.send_stanza(stanza)
    }

    pub fn send_reactions(
        &self,
        id: MessageId,
        to: &BareJid,
        reactions: &[impl AsRef<str>],
    ) -> Result<()> {
        let mut stanza = Stanza::new_message(
            Some("chat"),
            Some(&self.ctx.generate_id()),
            Some(&to.to_string()),
        );

        let mut reactions_node = Stanza::new();
        reactions_node.set_name("reactions")?;
        reactions_node.set_id(id.as_ref())?;
        reactions_node.set_ns(Namespace::Reactions)?;

        for reaction in reactions {
            let mut reaction_node = Stanza::new();
            reaction_node.set_name("reaction")?;
            reaction_node.add_child(Stanza::new_text_node(reaction)?)?;
            reactions_node.add_child(reaction_node)?;
        }
        stanza.add_child(reactions_node)?;

        self.ctx.send_stanza(stanza)
    }

    pub fn retract_message(&self, id: MessageId, to: &BareJid) -> Result<()> {
        let mut retraction_node = Stanza::new();
        retraction_node.set_name("retract")?;
        retraction_node.set_ns(Namespace::Retract)?;

        let mut fastening_node = Stanza::new();
        fastening_node.set_name("apply-to")?;
        fastening_node.set_id(id)?;
        fastening_node.set_ns(Namespace::Fasten)?;
        fastening_node.add_child(retraction_node)?;

        let mut fallback_node = Stanza::new();
        fallback_node.set_name("fallback")?;
        fallback_node.set_ns(Namespace::Fallback)?;

        let mut body_node = Stanza::new();
        body_node.set_name("body")?;
        body_node.add_child(Stanza::new_text_node("This person attempted to retract a previous message, but it's unsupported by your client.")?)?;

        let mut stanza = Stanza::new_message(
            Some("chat"),
            Some(&self.ctx.generate_id()),
            Some(&to.to_string()),
        );
        stanza.add_child(fastening_node)?;
        stanza.add_child(fallback_node)?;
        stanza.add_child(body_node)?;

        self.ctx.send_stanza(stanza)
    }

    pub fn set_message_carbons_enabled(&self, enabled: bool) -> Result<()> {
        let mut toggle_node = Stanza::new();
        if enabled {
            toggle_node.set_name("enable")?;
        } else {
            toggle_node.set_name("disable")?;
        }
        toggle_node.set_ns(Namespace::MessageCarbons)?;

        let mut stanza = Stanza::new_iq(Some("set"), Some(&self.ctx.generate_id()));
        stanza.set_attribute("from", self.ctx.jid.to_string())?;
        stanza.add_child(toggle_node)?;

        self.ctx.send_stanza(stanza)
    }
}
