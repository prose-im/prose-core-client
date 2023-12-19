// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::any::TypeId;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::task::Waker;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use jid::FullJid;
use minidom::Element;
use prose_wasm_utils::PinnedFuture;
use tracing::{error, warn};

use crate::client::builder::ClientBuilder;
use crate::client::module_context::ModuleContextInner;
use crate::client::{Event, ModuleLookup};
use crate::connector::{ConnectionError, ConnectionEvent};
use crate::mods;
use crate::mods::AnyModule;
use crate::util::{ModuleFuturePoll, XMPPElement};
use crate::Event as ClientEvent;

#[derive(Clone)]
pub struct Client {
    pub(super) inner: Arc<ClientInner>,
}

impl Debug for Client {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client").finish()
    }
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub async fn connect(
        &self,
        jid: &FullJid,
        password: impl AsRef<str>,
    ) -> Result<(), ConnectionError> {
        self.inner.clone().connect(jid, password).await
    }

    pub fn disconnect(&self) {
        self.inner.disconnect()
    }

    pub fn connected_jid(&self) -> Option<FullJid> {
        self.inner.context.jid.read().clone()
    }

    pub fn get_mod<M: AnyModule + Clone>(&self) -> M {
        self.inner.get_mod()
    }

    pub fn send_raw_stanza(&self, stanza: impl Into<Element>) -> Result<()> {
        self.inner.context.send_stanza(stanza)
    }
}

pub(super) struct ClientInner {
    pub context: Arc<ModuleContextInner>,
    pub mods: Arc<ModuleLookup>,
}

const TIMEOUT_DURATION: Duration = Duration::from_secs(15);

impl ClientInner {
    async fn connect(
        self: Arc<Self>,
        jid: &FullJid,
        password: impl AsRef<str>,
    ) -> Result<(), ConnectionError> {
        self.disconnect();

        *self.context.jid.write() = Some(jid.clone());

        let inner = self.clone();

        let connection = (self.context.connector_provider)()
            .connect(
                jid,
                password.as_ref(),
                Box::new(move |_, event| {
                    let inner = inner.clone();

                    Box::pin(async move { inner.handle_event(event).await }) as PinnedFuture<_>
                }),
            )
            .await?;

        self.context.connection.write().replace(connection);

        for (_, m) in self.mods.iter() {
            if let Err(err) = m.read().handle_connect() {
                println!("Encountered error in module {}", err);
            }
        }

        self.context
            .clone()
            .schedule_event(ClientEvent::Client(Event::Connected));

        Ok(())
    }

    fn disconnect(&self) {
        self.context.disconnect()
    }

    fn get_mod<M: AnyModule + Clone>(&self) -> M {
        let Some(entry) = self.mods.iter().find(|(k, _)| k == &TypeId::of::<M>()) else {
            panic!("Could not find requested module.")
        };
        entry.1.read().as_any().downcast_ref::<M>().unwrap().clone()
    }

    async fn handle_event(self: Arc<Self>, event: ConnectionEvent) {
        match event {
            ConnectionEvent::Disconnected { error } => self
                .context
                .clone()
                .schedule_event(ClientEvent::Client(Event::Disconnected { error })),
            ConnectionEvent::Stanza(stanza) => {
                Self::handle_stanza(&self.context, &self.mods, stanza)
            }
            ConnectionEvent::TimeoutTimer => Self::purge_expired_futures(&self.context),
            ConnectionEvent::PingTimer => {
                let ping = self.get_mod::<mods::Ping>();
                match ping.send_ping().await {
                    Ok(_) => (),
                    Err(err) => warn!("Failed to send ping. {}", err),
                }
            }
        }
    }

    fn handle_stanza(ctx: &ModuleContextInner, mods: &ModuleLookup, stanza: Element) {
        let elem = match XMPPElement::try_from_element(stanza) {
            Ok(None) => return,
            Ok(Some(elem)) => elem,
            Err(err) => {
                error!("Failed to parse stanza. {}", err);
                return;
            }
        };

        let mut wakers = Vec::<Waker>::new();
        let mut idx = 0;
        let mut pending_futures = ctx.mod_futures.lock();

        while idx < pending_futures.len() {
            let poll = pending_futures[idx].state.lock().handle_element(&elem);

            match poll {
                ModuleFuturePoll::Pending => idx += 1,
                ModuleFuturePoll::Ready(waker) => {
                    pending_futures.remove(idx);
                    if let Some(waker) = waker {
                        wakers.push(waker)
                    }
                }
            }
        }
        drop(pending_futures);

        for (_, m) in mods.iter() {
            if let Err(err) = m.read().handle_element(&elem) {
                error!("Encountered error in module {}", err);
            }
        }

        for waker in wakers {
            waker.wake()
        }
    }

    fn purge_expired_futures(ctx: &ModuleContextInner) {
        let mut pending_futures = ctx.mod_futures.lock();

        let now: SystemTime = ctx.time_provider.now().into();
        let mut wakers = Vec::<Waker>::new();
        let mut idx = 0;

        while idx < pending_futures.len() {
            if now.duration_since(pending_futures[idx].timestamp).unwrap() < TIMEOUT_DURATION {
                idx += 1
            } else {
                if let Some(waker) = pending_futures[idx].state.lock().fail_with_timeout() {
                    wakers.push(waker);
                }
                pending_futures.remove(idx);
            }
        }
        drop(pending_futures);

        for waker in wakers {
            waker.wake()
        }
    }
}

impl TryFrom<Arc<ModuleContextInner>> for Client {
    type Error = anyhow::Error;

    fn try_from(value: Arc<ModuleContextInner>) -> std::result::Result<Self, Self::Error> {
        let mods = value.mods.upgrade().ok_or(anyhow::format_err!(
            "Used module after client was released."
        ))?;

        Ok(Client {
            inner: Arc::new(ClientInner {
                context: value,
                mods,
            }),
        })
    }
}
