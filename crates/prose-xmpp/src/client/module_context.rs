// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::future::Future;
use std::string::ToString;
use std::sync::{Arc, Weak};
use std::time::SystemTime;

use anyhow::Result;
use jid::{BareJid, DomainPart, FullJid, Jid, NodePart, ResourcePart};
use minidom::Element;
use parking_lot::{Mutex, RwLock};
use prose_wasm_utils::PinnedFuture;
use xmpp_parsers::iq::Iq;

use crate::client::builder::UndefinedConnector;
use crate::client::{ConnectorProvider, EventHandler, ModuleLookup};
use crate::connector::Connection;
use crate::deps::{IDProvider, SystemTimeProvider, TimeProvider, UUIDProvider};
use crate::util::{ModuleFutureState, RequestError, RequestFuture};
use crate::Event;

#[derive(Clone)]
pub struct ModuleContext {
    pub(super) inner: Arc<ModuleContextInner>,
}

impl ModuleContext {
    pub(crate) fn send_iq(
        &self,
        iq: Iq,
    ) -> impl Future<Output = Result<Option<Element>, RequestError>> {
        let future = RequestFuture::new_iq_request(&iq.id);
        self.send_stanza_with_future(iq, future)
    }

    pub(crate) fn send_stanza_with_future<T: Send + 'static, U: 'static>(
        &self,
        stanza: impl Into<Element>,
        future: RequestFuture<T, U>,
    ) -> impl Future<Output = Result<U, RequestError>> {
        self.inner.mod_futures.lock().push(ModFutureStateEntry {
            state: future.state.clone(),
            timestamp: self.inner.time_provider.now().into(),
        });

        if let Err(err) = self.send_stanza(stanza) {
            return RequestFuture::failed(RequestError::Generic {
                msg: err.to_string(),
            });
        }

        future
    }

    pub(crate) fn send_stanza(&self, stanza: impl Into<Element>) -> Result<()> {
        let Some(conn) = &*self.inner.connection.read() else {
            return Ok(());
        };
        conn.send_stanza(stanza.into())
    }

    pub(crate) fn full_jid(&self) -> FullJid {
        self.inner
            .jid
            .read()
            .as_ref()
            .cloned()
            .unwrap_or(FullJid::from_parts(
                Some(&NodePart::new("placeholder").unwrap()),
                &DomainPart::new("prose.org").unwrap(),
                &ResourcePart::new("lib").unwrap(),
            ))
    }
    pub(crate) fn bare_jid(&self) -> BareJid {
        Jid::Full(self.full_jid()).into_bare()
    }

    pub(crate) fn server_jid(&self) -> BareJid {
        BareJid::from_parts(None, &self.full_jid().domain())
    }

    pub(crate) fn generate_id(&self) -> String {
        self.inner.id_provider.new_id()
    }

    pub(crate) fn schedule_event(&self, event: Event) {
        self.inner.clone().schedule_event(event)
    }

    pub(crate) fn disconnect(&self) {
        self.inner.disconnect();
    }
}

pub(super) struct ModuleContextInner {
    pub jid: RwLock<Option<FullJid>>,
    pub connector_provider: ConnectorProvider,
    pub connection: RwLock<Option<Box<dyn Connection>>>,
    pub event_handler: EventHandler,
    pub mods: Weak<ModuleLookup>,
    pub mod_futures: Mutex<Vec<ModFutureStateEntry>>,
    pub id_provider: Box<dyn IDProvider>,
    pub time_provider: Box<dyn TimeProvider>,
}

impl ModuleContextInner {
    #[cfg(not(feature = "test"))]
    pub(crate) fn schedule_event(self: Arc<Self>, event: Event) {
        let fut = (self.event_handler)(self.clone().try_into().unwrap(), event);
        prose_wasm_utils::spawn(fut);
    }

    #[cfg(feature = "test")]
    pub(crate) fn schedule_event(self: Arc<Self>, event: Event) {
        tokio::task::block_in_place(move || {
            let fut = (self.event_handler)(self.clone().try_into().unwrap(), event);
            tokio::runtime::Handle::current().block_on(async move { fut.await });
        });
    }

    pub(crate) fn disconnect(&self) {
        if let Some(conn) = self.connection.write().take() {
            conn.disconnect()
        }
    }
}

pub(super) struct ModFutureStateEntry {
    pub state: Arc<Mutex<dyn ModuleFutureState>>,
    pub timestamp: SystemTime,
}

impl Default for ModuleContext {
    fn default() -> Self {
        ModuleContext {
            inner: Arc::new(ModuleContextInner {
                connector_provider: Box::new(|| Box::new(UndefinedConnector {})),
                jid: RwLock::new(None),
                connection: Default::default(),
                event_handler: Box::new(|_, _| Box::pin(async {}) as PinnedFuture<_>),
                mods: Default::default(),
                mod_futures: Default::default(),
                id_provider: Box::new(UUIDProvider::new()),
                time_provider: Box::new(SystemTimeProvider::default()),
            }),
        }
    }
}
