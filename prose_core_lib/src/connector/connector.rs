use async_trait::async_trait;
use jid::FullJid;

use crate::{ConnectionError, ConnectionHandler, StanzaHandler, TimedHandler};

#[async_trait]
pub trait Connector {
    async fn connect(
        &self,
        config: ConnectionConfiguration,
    ) -> anyhow::Result<Box<dyn Connection>, ConnectionError>;
}

pub struct ConnectionConfiguration {
    pub jid: FullJid,
    pub password: String,
    pub connection_handler: ConnectionHandler,
    pub stanza_handler: StanzaHandler,
    pub timeout_handler: TimedHandler,
    pub ping_handler: TimedHandler,
}

pub trait Connection: Send + Sync {
    fn disconnect(&self);

    /// Send an XML stanza to the XMPP server.
    ///
    /// This is the main way to send data to the XMPP server. The function will terminate without
    /// action if the connection state is not CONNECTED.
    fn send_stanza(&self, stanza: libstrophe::Stanza);
}

impl ConnectionConfiguration {
    pub fn new(jid: FullJid, password: impl Into<String>) -> Self {
        ConnectionConfiguration {
            jid,
            password: password.into(),
            connection_handler: Box::new(|_, _| {}),
            stanza_handler: Box::new(|_, _| {}),
            timeout_handler: Box::new(|_| false),
            ping_handler: Box::new(|_| false),
        }
    }
}
