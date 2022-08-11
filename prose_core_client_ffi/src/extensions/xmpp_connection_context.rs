use crate::account::AccountObserver;
use crate::account::IDProvider;
use crate::connection::XMPPSender;
use crate::error::Result;
use jid::FullJid;
use libstrophe::Stanza;
use std::sync::Mutex;

pub struct XMPPExtensionContext {
    pub jid: FullJid,
    sender: Mutex<Box<dyn XMPPSender>>,
    id_provider: Box<dyn IDProvider>,
    pub observer: Box<dyn AccountObserver>,
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
            observer,
        }
    }

    pub fn send_stanza(&self, stanza: Stanza) -> Result<()> {
        self.sender.lock()?.send_stanza(stanza)
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
}
