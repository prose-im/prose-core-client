use std::any::TypeId;
use std::collections::BTreeMap;

use parking_lot::RwLock;

pub use builder::ClientBuilder;
pub use client::Client;
pub(crate) use module_context::ModuleContext;

use crate::connector::Connector;
use crate::mods::AnyModule;
use crate::util::PinnedFuture;
use crate::Event;

mod builder;
mod client;
mod module_context;

#[cfg(target_arch = "wasm32")]
pub type EventHandler = Box<dyn Fn(Client, Event) -> PinnedFuture<()>>;
#[cfg(not(target_arch = "wasm32"))]
pub type EventHandler = Box<dyn Fn(Client, Event) -> PinnedFuture<()> + Send + Sync>;

pub(super) type ModuleLookup = BTreeMap<TypeId, RwLock<Box<dyn AnyModule>>>;
pub type ConnectorProvider = fn() -> Box<dyn Connector>;
