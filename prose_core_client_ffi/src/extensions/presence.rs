use crate::error::Result;
use crate::extensions::{XMPPExtension, XMPPExtensionContext};
use crate::types;
use crate::types::presence::ShowKind;
use libstrophe::Stanza;
use std::sync::Arc;

pub struct Presence {
    ctx: Arc<XMPPExtensionContext>,
}

impl Presence {
    pub fn new(ctx: Arc<XMPPExtensionContext>) -> Self {
        Presence { ctx }
    }
}

impl XMPPExtension for Presence {
    fn handle_connect(&self) -> Result<()> {
        // After establishing a session, a client SHOULD send initial presence to the server
        // in order to signal its availability for communications. As defined herein, the initial
        // presence stanza (1) MUST possess no 'to' address (signalling that it is meant to be
        // broadcast by the server on behalf of the client) and (2) MUST possess no 'type' attribute
        // (signalling the user's availability). After sending initial presence, an active resource is
        // said to be an "available resource".
        self.ctx.send_stanza(Stanza::new_presence())
    }

    fn handle_presence_stanza(&self, stanza: &Stanza) -> Result<()> {
        let presence: types::presence::Presence = stanza.try_into()?;
        self.ctx.observer.did_receive_presence(presence);
        Ok(())
    }
}

impl Presence {
    pub fn send_presence(&self, show: Option<ShowKind>, status: Option<&str>) -> Result<()> {
        let mut presence_stanza = Stanza::new_presence();

        if let Some(show) = show {
            let mut show_node = Stanza::new();
            show_node.set_name("show")?;

            let mut text_node = Stanza::new();
            text_node.set_text(show.to_string())?;
            show_node.add_child(text_node)?;

            presence_stanza.add_child(show_node)?;
        }

        if let Some(status) = status {
            let mut status_node = Stanza::new();
            status_node.set_name("status")?;

            let mut text_node = Stanza::new();
            text_node.set_text(status)?;
            status_node.add_child(text_node)?;

            presence_stanza.add_child(status_node)?;
        }

        self.ctx.send_stanza(presence_stanza)?;
        Ok(())
    }
}
