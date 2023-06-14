use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use jid::FullJid;

use crate::connector::Connection;
use crate::dependencies::{IDProvider, TimeProvider};
use crate::modules::request_future::{RequestError, RequestFuture, RequestFutureState};
use crate::stanza::{iq, StanzaBase, IQ};

pub struct PendingRequest {
    scheduled_at: SystemTime,
    timeout: Duration,
    state: ConnectionRequest,
}

enum ConnectionRequest {
    Future(Arc<Mutex<RequestFutureState>>),
    Callback(Box<dyn FnOnce(&Context, Result<IQ, RequestError>) + Send>),
}

pub struct Context<'a> {
    pub jid: &'a FullJid,
    connection: &'a dyn Connection,
    id_provider: &'a dyn IDProvider,
    time_provider: &'a dyn TimeProvider,
    pending_request_futures: &'a Mutex<HashMap<iq::Id, PendingRequest>>,
}

impl<'a> Context<'a> {
    pub fn new(
        jid: &'a FullJid,
        sender: &'a dyn Connection,
        id_provider: &'a dyn IDProvider,
        time_provider: &'a dyn TimeProvider,
        pending_request_futures: &'a Mutex<HashMap<iq::Id, PendingRequest>>,
    ) -> Self {
        Context {
            jid,
            connection: sender,
            id_provider,
            time_provider,
            pending_request_futures,
        }
    }

    pub fn send_stanza(&self, stanza: impl StanzaBase) {
        self.connection.send_stanza(stanza.stanza_owned())
    }

    pub async fn send_iq(&self, iq: IQ<'_>) -> Result<IQ, RequestError> {
        self.send_iq_with_timeout(iq, Duration::from_secs(10)).await
    }

    pub async fn send_iq_with_timeout(
        &self,
        iq: IQ<'_>,
        timeout: Duration,
    ) -> Result<IQ, RequestError> {
        let id = iq
            .id()
            .expect("Missing id for IQ stanza. Did you forget to set one?");

        let fut = RequestFuture::new();
        self.pending_request_futures.lock().unwrap().insert(
            id,
            PendingRequest {
                scheduled_at: self.time_provider.now(),
                timeout,
                state: ConnectionRequest::Future(fut.state.clone()),
            },
        );
        self.send_stanza(iq);

        Ok(fut.await?)
    }

    pub fn send_iq_with_timeout_cb<F>(
        &self,
        iq: IQ<'_>,
        timeout: Duration,
        cb: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(&Context, Result<IQ, RequestError>) + Send + 'static,
    {
        let id = iq
            .id()
            .expect("Missing id for IQ stanza. Did you forget to set one?");

        self.pending_request_futures.lock().unwrap().insert(
            id,
            PendingRequest {
                scheduled_at: self.time_provider.now(),
                timeout,
                state: ConnectionRequest::Callback(Box::new(cb)),
            },
        );
        self.send_stanza(iq);
        Ok(())
    }

    pub fn generate_id(&self) -> String {
        self.id_provider.new_id()
    }
}

impl<'a> Context<'a> {
    pub(crate) fn purge_pending_futures(&self) -> anyhow::Result<()> {
        let mut result_handlers = self.pending_request_futures.lock().unwrap();

        if result_handlers.len() < 1 {
            return Ok(());
        }

        let now = self.time_provider.now();
        let mut keys_to_remove = HashSet::new();

        for (k, v) in result_handlers.iter_mut() {
            if now.duration_since(v.scheduled_at)? < v.timeout {
                continue;
            }
            keys_to_remove.insert(k.clone());
        }

        for ref key in keys_to_remove {
            let entry = result_handlers.remove(key);
            if let Some(entry) = entry {
                entry.state.fail(self, RequestError::TimedOut)
            }
        }

        Ok(())
    }

    pub(crate) fn handle_iq_result(
        &self,
        id: &iq::Id,
        payload: Result<IQ<'static>, RequestError>,
    ) -> anyhow::Result<()> {
        let mut result_handlers = self.pending_request_futures.lock().unwrap();
        let entry = result_handlers.remove(id);

        if let Some(entry) = entry {
            match payload {
                Ok(value) => entry.state.fulfill(self, value),
                Err(err) => entry.state.fail(self, err),
            }
        }

        Ok(())
    }

    pub(crate) fn disconnect(&self) {
        self.connection.disconnect()
    }
}

impl ConnectionRequest {
    pub(crate) fn fulfill(self, ctx: &Context, value: IQ<'static>) {
        match self {
            Self::Future(fut) => fut.lock().unwrap().fulfill(value),
            Self::Callback(cb) => (cb)(ctx, Ok(value)),
        }
    }

    pub(crate) fn fail(self, ctx: &Context, error: RequestError) {
        match self {
            Self::Future(fut) => fut.lock().unwrap().fail(error),
            Self::Callback(cb) => (cb)(ctx, Err(error)),
        }
    }
}
