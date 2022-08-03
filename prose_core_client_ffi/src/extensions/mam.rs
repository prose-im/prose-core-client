use crate::error::{Error, Result, StanzaParseError};
use crate::extensions::{XMPPExtension, XMPPExtensionContext};
use crate::helpers::StanzaExt;
use crate::types::forwarded_message::ForwardedMessage;
use crate::types::mam::{Fin, MAMPreferences, Preferences};
use crate::types::namespace::Namespace;
use crate::MessageId;
use jid::BareJid;
use libstrophe::Stanza;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

pub struct MAM {
    ctx: Arc<XMPPExtensionContext>,
    pending_requests: Mutex<HashMap<String, MessageRequest>>,
}

impl MAM {
    pub fn new(ctx: Arc<XMPPExtensionContext>) -> Self {
        MAM {
            ctx,
            pending_requests: Mutex::new(HashMap::new()),
        }
    }
}

impl XMPPExtension for MAM {
    fn handle_message_stanza(&self, stanza: &Stanza) -> Result<()> {
        let result_node = match stanza.get_child_by_name_and_ns("result", Namespace::MAM2) {
            None => return Ok(()),
            Some(node) => node,
        };

        let query_id = match result_node.get_attribute("queryid") {
            None => return Ok(()),
            Some(query_id) => query_id,
        };

        let message = result_node
            .get_child_by_name_and_ns("forwarded", Namespace::Forward)
            .ok_or(Error::StanzaParseError {
                error: StanzaParseError::missing_child_node("forwarded", stanza),
            })
            .and_then(|n| ForwardedMessage::try_from(n.deref()))?;

        let mut locked_hash_map = self.pending_requests.lock()?;
        let pending_request = match locked_hash_map.get_mut(query_id) {
            None => return Ok(()),
            Some(pending_request) => pending_request,
        };

        pending_request.messages.push(message);
        Ok(())
    }

    fn handle_iq_stanza(&self, stanza: &Stanza) -> Result<()> {
        if let Some(prefs_node) = stanza.get_child_by_name_and_ns("prefs", Namespace::MAM2) {
            self.ctx.observer.did_receive_archiving_preferences(
                Preferences::try_from(prefs_node.deref())?.into(),
            );
            return Ok(());
        }

        if let Some(fin_node) = stanza.get_child_by_name_and_ns("fin", Namespace::MAM2) {
            self.handle_fin_node(fin_node.deref())?;
        }

        Ok(())
    }
}

impl MAM {
    fn handle_fin_node(&self, fin_stanza: &Stanza) -> Result<()> {
        let fin = Fin::try_from(fin_stanza)?;

        let query_id = match fin.query_id {
            None => return Ok(()),
            Some(query_id) => query_id,
        };

        let pending_request = match self.pending_requests.lock()?.remove(&query_id) {
            None => return Ok(()),
            Some(pending_request) => pending_request,
        };

        self.ctx.observer.did_receive_messages_in_chat(
            pending_request.id,
            pending_request.jid,
            pending_request.messages,
            fin.complete,
        );

        Ok(())
    }
}

impl MAM {
    pub fn load_archiving_preferences(&self) -> Result<()> {
        let mut prefs = Stanza::new();
        prefs.set_name("prefs")?;
        prefs.set_ns(Namespace::MAM2)?;

        let mut iq = Stanza::new_iq(Some("get"), Some(&self.ctx.generate_id()));
        iq.add_child(prefs)?;

        self.ctx.send_stanza(iq)
    }

    pub fn set_archiving_preferences(&self, preferences: &MAMPreferences) -> Result<()> {
        let mut iq = Stanza::new_iq(Some("set"), Some(&self.ctx.generate_id()));
        iq.add_child(preferences.try_into()?)?;
        self.ctx.send_stanza(iq)
    }

    pub fn load_messages_in_chat(
        &self,
        request_id: &str,
        jid: &BareJid,
        before: Option<MessageId>,
    ) -> Result<()> {
        let query_id = self.ctx.generate_id();

        let mut x = Stanza::new();
        x.set_name("x")?;
        x.set_ns(Namespace::DataForms)?;
        x.set_attribute("type", "submit")?;
        x.add_child(Stanza::new_form_field(
            "FORM_TYPE",
            Namespace::MAM2,
            Some("hidden"),
        )?)?;
        x.add_child(Stanza::new_form_field("with", jid.to_string(), None)?)?;

        if let Some(before) = before {
            x.add_child(Stanza::new_form_field("before-id", before, None)?)?;
        }

        let mut flip = Stanza::new();
        flip.set_name("flip-page")?;

        let mut query = Stanza::new_query(Namespace::MAM2, Some(&query_id))?;
        query.add_child(x)?;
        query.add_child(flip)?;

        let mut iq = Stanza::new_iq(Some("set"), Some(&request_id));
        iq.add_child(query)?;

        let request = MessageRequest::new(request_id, jid);
        self.pending_requests.lock()?.insert(query_id, request);

        self.ctx.send_stanza(iq)
    }
}

struct MessageRequest {
    id: String,
    jid: BareJid,
    messages: Vec<ForwardedMessage>,
}

impl MessageRequest {
    fn new(id: impl AsRef<str>, jid: &BareJid) -> Self {
        MessageRequest {
            id: id.as_ref().to_owned(),
            jid: jid.clone(),
            messages: Vec::new(),
        }
    }
}
