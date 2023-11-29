// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::any::TypeId;

use parking_lot::RwLock;

pub use builder::ClientBuilder;
pub use client::Client;
pub(crate) use module_context::ModuleContext;
use prose_wasm_utils::PinnedFuture;

use crate::connector::ConnectionError;
use crate::connector::Connector;
use crate::mods::AnyModule;
use crate::Event as ClientEvent;

mod builder;
mod client;
mod module_context;

#[cfg(target_arch = "wasm32")]
pub type EventHandler = Box<dyn Fn(Client, ClientEvent) -> PinnedFuture<()>>;
#[cfg(not(target_arch = "wasm32"))]
pub type EventHandler = Box<dyn Fn(Client, ClientEvent) -> PinnedFuture<()> + Send + Sync>;

pub(super) type ModuleLookup = Vec<(TypeId, RwLock<Box<dyn AnyModule>>)>;

#[cfg(target_arch = "wasm32")]
pub type ConnectorProvider = Box<dyn Fn() -> Box<dyn Connector>>;
#[cfg(not(target_arch = "wasm32"))]
pub type ConnectorProvider = Box<dyn Fn() -> Box<dyn Connector> + Send + Sync>;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Connected,
    Disconnected { error: Option<ConnectionError> },
}
