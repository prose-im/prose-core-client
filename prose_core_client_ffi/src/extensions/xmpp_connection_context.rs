use crate::connection::XMPPSender;
use crate::error::Result;
use crate::AccountObserver;
use libstrophe::Stanza;
use std::sync::Mutex;

pub struct XMPPExtensionContext {
    sender: Mutex<Box<dyn XMPPSender>>,
    pub observer: Box<dyn AccountObserver>,
}

impl XMPPExtensionContext {
    pub fn new(sender: Box<dyn XMPPSender>, observer: Box<dyn AccountObserver>) -> Self {
        XMPPExtensionContext {
            sender: Mutex::new(sender),
            observer,
        }
    }

    pub fn send_stanza(&self, stanza: Stanza) -> Result<()> {
        self.sender.lock()?.send_stanza(stanza)
    }
}

impl XMPPExtensionContext {
    pub fn replace_sender(&self, sender: Box<dyn XMPPSender>) -> Result<()> {
        let mut current_sender = self.sender.lock()?;
        *current_sender = sender;
        Ok(())
    }
}
