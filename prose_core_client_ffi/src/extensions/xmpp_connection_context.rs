// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::account::AccountObserver;
use crate::account::IDProvider;
use crate::connection::XMPPSender;
use crate::error::Result;
use jid::FullJid;
use libstrophe::{Stanza, StanzaRef};
use std::collections::HashMap;
use std::sync::Mutex;
use strum_macros::{Display, EnumString};

pub type IQResultHandler = Box<dyn FnOnce(Result<StanzaRef, StanzaRef>) -> Result<()> + Send>;

pub struct XMPPExtensionContext {
    pub jid: FullJid,
    sender: Mutex<Box<dyn XMPPSender>>,
    id_provider: Box<dyn IDProvider>,
    iq_result_handlers: Mutex<HashMap<String, IQResultHandler>>,
    pub observer: Box<dyn AccountObserver>,
}

#[derive(Debug, PartialEq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum IQKind {
    Get,
    Set,
}

impl XMPPExtensionContext {
    pub fn new(
        jid: FullJid,
        sender: Box<dyn XMPPSender>,
        id_provider: Box<dyn IDProvider>,
        observer: Box<dyn AccountObserver>,
    ) -> Self {
        XMPPExtensionContext {
            jid,
            sender: Mutex::new(sender),
            id_provider,
            iq_result_handlers: Mutex::new(HashMap::new()),
            observer,
        }
    }

    pub fn send_stanza(&self, stanza: Stanza) -> Result<()> {
        self.sender.lock()?.send_stanza(stanza)
    }

    pub fn send_iq(
        &self,
        kind: IQKind,
        to: Option<&str>,
        payload: Stanza,
        handler: IQResultHandler,
    ) -> Result<()> {
        let id = self.generate_id();

        let mut iq = Stanza::new_iq(Some(&kind.to_string()), Some(&id));
        if let Some(to) = to {
            iq.set_to(to)?;
        }
        iq.add_child(payload)?;

        self.iq_result_handlers.lock()?.insert(id, handler);
        self.send_stanza(iq)
    }

    pub fn generate_id(&self) -> String {
        self.id_provider.new_id()
    }
}

impl XMPPExtensionContext {
    pub fn replace_sender(&self, sender: Box<dyn XMPPSender>) -> Result<()> {
        let mut current_sender = self.sender.lock()?;
        *current_sender = sender;
        Ok(())
    }

    pub fn handle_iq_result(&self, id: &str, payload: Result<StanzaRef, StanzaRef>) -> Result<()> {
        let mut result_handlers = self.iq_result_handlers.lock()?;
        let handler = result_handlers.remove(id);
        drop(result_handlers);

        if let Some(handler) = handler {
            handler(payload)?;
        }

        Ok(())
    }
}
