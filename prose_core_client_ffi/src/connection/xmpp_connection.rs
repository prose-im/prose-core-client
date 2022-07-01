use crate::error::{Error, Result};
use libstrophe::Stanza;
use strum_macros::Display;

#[derive(Debug, Display)]
pub enum ConnectionEvent {
    Connect,
    Disconnect(Option<Error>),
}

pub type StanzaHandler = Box<dyn FnMut(&Stanza) + Send>;
pub type ConnectionHandler = Box<dyn FnMut(&ConnectionEvent) + Send>;

pub trait XMPPConnection {
    fn connect(self: Box<Self>) -> Result<Box<dyn XMPPSender>>;

    fn set_connection_handler(&mut self, handler: ConnectionHandler);
    fn set_stanza_handler(&mut self, handler: StanzaHandler);
}

pub trait XMPPSender: Send {
    /// Send an XML stanza to the XMPP server.
    ///
    /// This is the main way to send data to the XMPP server. The function will terminate without
    /// action if the connection state is not CONNECTED.
    fn send_stanza(&self, stanza: Stanza) -> Result<()>;
}
