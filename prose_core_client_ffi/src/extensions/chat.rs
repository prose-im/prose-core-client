use crate::error::Result;
use crate::extensions::{XMPPExtension, XMPPExtensionContext};
use crate::types::namespace::Namespace;
use crate::{ChatState, Message};
use jid::BareJid;
use libstrophe::Stanza;
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
    fn handle_message_stanza(&self, stanza: &Stanza) -> Result<()> {
        let message: Message = stanza.try_into()?;
        self.ctx.observer.did_receive_message(message);
        Ok(())
    }
}

impl Chat {
    pub fn send_message(
        &self,
        id: &str,
        to: &BareJid,
        body: &str,
        chat_state: Option<ChatState>,
    ) -> Result<()> {
        let mut stanza = Stanza::new_message(Some("chat"), Some(id), Some(&to.to_string()));
        stanza.set_body(&body.to_string())?;

        if let Some(chat_state) = chat_state {
            let mut chat_state_node = Stanza::new();
            chat_state_node.set_name(chat_state.to_string())?;
            chat_state_node.set_ns(Namespace::ChatStates)?;
            stanza.add_child(chat_state_node)?;
        }

        self.ctx.send_stanza(stanza)
    }

    pub fn update_message(&self, id: &str, new_id: &str, to: &BareJid, body: &str) -> Result<()> {
        let mut stanza = Stanza::new_message(None, Some(new_id), Some(&to.to_string()));
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
}
