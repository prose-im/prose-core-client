// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::FullJid;
use minidom::Element;
use prose_wasm_utils::{PinnedFuture, SendUnlessWasm, SyncUnlessWasm};
use secrecy::Secret;

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum ConnectionError {
    #[error("Timed out")]
    TimedOut,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("{msg:?}")]
    Generic { msg: String },
}

#[cfg(target_arch = "wasm32")]
pub type ConnectionEventHandler =
    Box<dyn Fn(Box<dyn Connection>, ConnectionEvent) -> PinnedFuture<()>>;
#[cfg(not(target_arch = "wasm32"))]
pub type ConnectionEventHandler =
    Box<dyn Fn(Box<dyn Connection>, ConnectionEvent) -> PinnedFuture<()> + Send + Sync>;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Connector: SendUnlessWasm + SyncUnlessWasm {
    async fn connect(
        &self,
        jid: &FullJid,
        password: Secret<String>,
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
