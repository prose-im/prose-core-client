use crate::error::Result;
use libstrophe::Stanza;

mod chat;
mod debug;
mod mam;
mod presence;
mod roster;
mod xmpp_connection_context;

pub(crate) use chat::Chat;
pub(crate) use debug::Debug;
pub(crate) use mam::MAM;
pub(crate) use presence::Presence;
pub(crate) use roster::Roster;
pub(crate) use xmpp_connection_context::XMPPExtensionContext;

#[allow(unused_variables)]
pub(crate) trait XMPPExtension: Send + Sync {
    fn handle_connect(&self) -> Result<()> {
        Ok(())
    }
    fn handle_disconnect(&self) -> Result<()> {
        Ok(())
    }

    fn handle_presence_stanza(&self, stanza: &Stanza) -> Result<()> {
        Ok(())
    }
    fn handle_message_stanza(&self, stanza: &Stanza) -> Result<()> {
        Ok(())
    }
    fn handle_iq_stanza(&self, stanza: &Stanza) -> Result<()> {
        Ok(())
    }
}
