use std::future::Future;
use std::string::ToString;
use std::sync::{Arc, Weak};
use std::time::SystemTime;

use anyhow::Result;
use jid::{BareJid, FullJid};
use minidom::Element;
use parking_lot::{Mutex, RwLock};
use xmpp_parsers::iq::Iq;

use crate::client::builder::UndefinedConnector;
use crate::client::{ConnectorProvider, EventHandler, ModuleLookup};
use crate::connector::Connection;
use crate::deps::{IDProvider, SystemTimeProvider, TimeProvider, UUIDProvider};
use crate::util::{ModuleFutureState, PinnedFuture, RequestError, RequestFuture};
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
        self.send_iq_with_future(iq, future)
    }

    pub(crate) fn send_iq_with_future<T: Send + 'static, U: 'static>(
        &self,
        iq: Iq,
        future: RequestFuture<T, U>,
    ) -> impl Future<Output = Result<U, RequestError>> {
        self.inner.mod_futures.lock().push(ModFutureStateEntry {
            state: future.state.clone(),
            timestamp: self.inner.time_provider.now(),
        });

        if let Err(err) = self.send_stanza(iq) {
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
            .map(Clone::clone)
            .unwrap_or(FullJid::new("placeholder", "prose.org", "lib"))
    }
    pub(crate) fn bare_jid(&self) -> BareJid {
        self.full_jid().into()
    }

    pub(crate) fn generate_id(&self) -> String {
        self.inner.id_provider.new_id()
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn schedule_event(&self, event: Event) {
        let fut = (self.inner.event_handler)(self.inner.clone().try_into().unwrap(), event);
        wasm_bindgen_futures::spawn_local(async move { fut.await });
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn schedule_event(&self, event: Event) {
        let fut = (self.inner.event_handler)(self.inner.clone().try_into().unwrap(), event);
        tokio::spawn(async move { fut.await });
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
                time_provider: Box::new(SystemTimeProvider::new()),
            }),
        }
    }
}
