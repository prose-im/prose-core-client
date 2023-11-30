// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::any::TypeId;
use std::future::Future;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use jid::FullJid;
use minidom::Element;
use parking_lot::RwLock;
use prose_wasm_utils::{PinnedFuture, SendUnlessWasm, SyncUnlessWasm};

use crate::client::client::ClientInner;
use crate::client::module_context::ModuleContextInner;
use crate::client::{ConnectorProvider, EventHandler, ModuleContext, ModuleLookup};
use crate::connector::{Connection, ConnectionError, ConnectionEventHandler, Connector};
use crate::deps::{IDProvider, SystemTimeProvider, TimeProvider, UUIDProvider};
use crate::mods::AnyModule;
use crate::{mods, Client, Event};

pub struct UndefinedConnector {}
pub struct UndefinedConnection {}

pub struct ClientBuilder {
    connector_provider: ConnectorProvider,
    mods: ModuleLookup,
    id_provider: Box<dyn IDProvider>,
    time_provider: Box<dyn TimeProvider>,
    event_handler: EventHandler,
}

impl ClientBuilder {
    pub(super) fn new() -> Self {
        ClientBuilder {
            connector_provider: Box::new(|| Box::new(UndefinedConnector {})),
            mods: Default::default(),
            id_provider: Box::new(UUIDProvider::new()),
            time_provider: Box::new(SystemTimeProvider::default()),
            event_handler: Box::new(|_, _| Box::pin(async {}) as PinnedFuture<_>),
        }
        // Order matters…
        .add_mod(mods::Bookmark2::default())
        .add_mod(mods::Bookmark::default())
        .add_mod(mods::Status::default())
        .add_mod(mods::Caps::default())
        .add_mod(mods::MUC::default())
        .add_mod(mods::Chat::default())
        .add_mod(mods::MAM::default())
        .add_mod(mods::Profile::default())
        .add_mod(mods::PubSub::default())
        .add_mod(mods::Roster::default())
    }

    pub fn set_connector_provider(self, connector_provider: ConnectorProvider) -> Self {
        ClientBuilder {
            connector_provider,
            mods: self.mods,
            id_provider: self.id_provider,
            time_provider: self.time_provider,
            event_handler: self.event_handler,
        }
    }

    pub fn set_event_handler<T>(
        self,
        handler: impl Fn(Client, Event) -> T + SendUnlessWasm + SyncUnlessWasm + 'static,
    ) -> Self
    where
        T: Future<Output = ()> + SendUnlessWasm + 'static,
    {
        ClientBuilder {
            connector_provider: self.connector_provider,
            mods: self.mods,
            id_provider: self.id_provider,
            time_provider: self.time_provider,
            event_handler: Box::new(move |client, event| {
                let fut = handler(client, event);
                Box::pin(async move { fut.await }) as PinnedFuture<_>
            }),
        }
    }

    pub fn set_id_provider<P: IDProvider + 'static>(mut self, id_provider: P) -> Self {
        self.id_provider = Box::new(id_provider);
        self
    }

    pub fn set_time_provider<T: TimeProvider + 'static>(mut self, time_provider: T) -> Self {
        self.time_provider = Box::new(time_provider);
        self
    }

    pub fn build(self) -> Client {
        let mut mods = self.mods;
        mods.push((
            TypeId::of::<mods::Ping>(),
            RwLock::new(Box::new(mods::Ping::default())),
        ));

        let mods = Arc::new(mods);

        let context_inner = Arc::new(ModuleContextInner {
            connector_provider: self.connector_provider,
            jid: RwLock::new(None),
            connection: Default::default(),
            mods: Arc::downgrade(&mods),
            mod_futures: Default::default(),
            id_provider: self.id_provider,
            time_provider: self.time_provider,
            event_handler: self.event_handler,
        });

        for (_, m) in mods.iter() {
            m.write().register_with(ModuleContext {
                inner: context_inner.clone(),
            });
        }

        Client {
            inner: Arc::new(ClientInner {
                mods: mods.clone(),
                context: context_inner,
            }),
        }
    }
}

impl ClientBuilder {
    fn add_mod<M: AnyModule + Clone + 'static>(mut self, m: M) -> Self {
        self.mods
            .push((TypeId::of::<M>(), RwLock::new(Box::new(m))));
        self
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl Connector for UndefinedConnector {
    async fn connect(
        &self,
        _jid: &FullJid,
        _password: &str,
        _event_handler: ConnectionEventHandler,
    ) -> Result<Box<dyn Connection>, ConnectionError> {
        panic!("Client doesn't have a connector. Provide one before calling connect()")
    }
}

impl Connection for UndefinedConnection {
    fn send_stanza(&self, _stanza: Element) -> Result<()> {
        panic!("Calling send_stanza on PlaceholderConnection is illegal.")
    }

    fn disconnect(&self) {
        panic!("Calling disconnect on PlaceholderConnection is illegal.")
    }
}
