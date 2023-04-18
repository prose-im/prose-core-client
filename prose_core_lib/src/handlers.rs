use libstrophe::Stanza;
use strum_macros::Display;

use crate::connector::Connection;
use crate::ConnectionError;

#[derive(Debug, Display, Clone)]
pub enum ConnectionEvent {
    Connect,
    Disconnect { error: ConnectionError },
}

pub type StanzaHandler = Box<dyn FnMut(&dyn Connection, &Stanza) + Send>;
pub type ConnectionHandler = Box<dyn FnMut(&dyn Connection, &ConnectionEvent) + Send>;
pub type TimedHandler = Box<dyn FnMut(&dyn Connection) -> bool + Send>;
