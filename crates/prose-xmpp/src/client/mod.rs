use std::any::TypeId;
use std::collections::BTreeMap;

use parking_lot::RwLock;

use crate::connector::ConnectionError;
use crate::connector::Connector;
use crate::mods::AnyModule;
use crate::util::PinnedFuture;
use crate::Event as ClientEvent;
pub use builder::ClientBuilder;
pub use client::Client;
pub(crate) use module_context::ModuleContext;

mod builder;
mod client;
mod module_context;

#[cfg(target_arch = "wasm32")]
pub type EventHandler = Box<dyn Fn(Client, ClientEvent) -> PinnedFuture<()>>;
#[cfg(not(target_arch = "wasm32"))]
pub type EventHandler = Box<dyn Fn(Client, ClientEvent) -> PinnedFuture<()> + Send + Sync>;

pub(super) type ModuleLookup = BTreeMap<TypeId, RwLock<Box<dyn AnyModule>>>;

#[cfg(target_arch = "wasm32")]
pub type ConnectorProvider = Box<dyn Fn() -> Box<dyn Connector>>;
#[cfg(not(target_arch = "wasm32"))]
pub type ConnectorProvider = Box<dyn Fn() -> Box<dyn Connector> + Send + Sync>;

#[derive(Debug, Clone)]
pub enum Event {
    Connected,
    Disconnected { error: Option<ConnectionError> },
}
