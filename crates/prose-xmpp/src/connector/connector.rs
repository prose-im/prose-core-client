use crate::SendUnlessWasm;
use anyhow::Result;
use async_trait::async_trait;
use jid::FullJid;
use minidom::Element;

use crate::util::PinnedFuture;

#[derive(Debug, thiserror::Error, Clone)]
pub enum ConnectionError {
    #[error("Timed out")]
    TimedOut,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("{msg:?}")]
    Generic { msg: String },
}

#[cfg(target_arch = "wasm32")]
pub type ConnectionEventHandler = Box<dyn Fn(&dyn Connection, ConnectionEvent) -> PinnedFuture<()>>;
#[cfg(not(target_arch = "wasm32"))]
pub type ConnectionEventHandler =
    Box<dyn Fn(&dyn Connection, ConnectionEvent) -> PinnedFuture<()> + Send + Sync>;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Connector: SendUnlessWasm {
    async fn connect(
        &self,
        jid: &FullJid,
        password: &str,
        event_handler: ConnectionEventHandler,
    ) -> Result<Box<dyn Connection>, ConnectionError>;
}

#[derive(Debug)]
pub enum ConnectionEvent {
    Disconnected { error: Option<ConnectionError> },
    Stanza(Element),
    TimeoutTimer,
    PingTimer,
}

#[cfg(target_arch = "wasm32")]
pub trait Connection {
    fn send_stanza(&self, stanza: Element) -> Result<()>;
    fn disconnect(&self);
}

#[cfg(not(target_arch = "wasm32"))]
pub trait Connection: Send + Sync {
    fn send_stanza(&self, stanza: Element) -> Result<()>;
    fn disconnect(&self);
}