use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use spin_sleep::SpinSleeper;
use tracing::{debug, info};
use uuid::Uuid;

use crate::connector::{Connection, ConnectionConfiguration, Connector};
use crate::{ConnectionError, ConnectionEvent};

use super::{ConnectionId, ConnectionMessage, LibstropheConnection};

static CONTEXT: Lazy<Mutex<Context>> =
    Lazy::new(|| Mutex::new(Context::new().expect("Could not start thread.")));

pub(crate) enum ContextMessage {
    Connect(ConnectionId, libstrophe::Context<'static, 'static>),
    Stop(ConnectionId),
}

pub struct LibstropheConnector {}

impl Default for LibstropheConnector {
    fn default() -> Self {
        LibstropheConnector {}
    }
}

#[async_trait]
impl Connector for LibstropheConnector {
    async fn connect(
        &self,
        config: ConnectionConfiguration,
    ) -> anyhow::Result<Box<dyn Connection>, ConnectionError> {
        let fut = ConnectionFuture::new();

        let mut config = config;
        {
            let mut fut_state = Some(fut.state.clone());
            config.connection_handler = Box::new(move |_, event| {
                if let Some(fut_state) = fut_state.take() {
                    let mut state = fut_state.lock().unwrap();
                    state.connection_event = Some(event.clone());

                    if let Some(waker) = state.waker.take() {
                        waker.wake();
                    }
                }
            });
        }

        let connection = self
            .inner_connect(config)
            .map_err(|e| ConnectionError::Generic { msg: e.to_string() })?;

        match fut.await {
            ConnectionEvent::Connect => Ok(Box::new(connection)),
            ConnectionEvent::Disconnect { error } => Err(error),
        }
    }
}

impl LibstropheConnector {
    fn inner_connect(
        &self,
        config: ConnectionConfiguration,
    ) -> anyhow::Result<LibstropheConnection> {
        let connection_id: ConnectionId = Uuid::new_v4().to_string().into();
        let (tx, rx) = channel::<ConnectionMessage>();

        let ctx = CONTEXT.lock().unwrap();

        let strophe_ctx = config.configure_and_connect(
            connection_id.clone(),
            libstrophe::Connection::new(libstrophe::Context::new_with_default_logger()),
            tx.clone(),
            ctx.sender.clone(),
            rx,
        )?;

        ctx.sender
            .send(ContextMessage::Connect(connection_id.clone(), strophe_ctx))
            .unwrap();

        let conn = LibstropheConnection::new(connection_id, tx, ctx.sender.clone());

        return Ok(conn);
    }
}

struct Context {
    sender: Sender<ContextMessage>,
    _thread: JoinHandle<()>,
}

impl Context {
    fn new() -> anyhow::Result<Self> {
        let (sender, receiver) = channel::<ContextMessage>();
        let mut contexts: HashMap<ConnectionId, libstrophe::Context> = HashMap::new();
        let event_loop_timeout = Duration::from_millis(100);
        let sleeper = SpinSleeper::default();

        info!("Starting XMPP polling thread…");
        let thread = thread::Builder::new()
            .name("org.prose.xmpp-thread".to_string())
            .spawn(move || loop {
                match receiver.try_recv() {
                    Ok(ContextMessage::Connect(connection_id, ctx)) => {
                        if let Some(former_ctx) = contexts.insert(connection_id, ctx) {
                            former_ctx.stop();
                        }
                    },
                    Ok(ContextMessage::Stop(connection_id)) => {
                        match contexts.remove(&connection_id) {
                            Some(ctx) => {
                                debug!(
                                    "Cleaning up context for connection with id {:?}. Remaining contexts {}.",
                                    connection_id, contexts.len()
                                );
                                ctx.stop();
                            }
                            None => {
                                debug!(
                                    "Could clean up connection with id {:?}. No such connection.",
                                    connection_id
                                )
                            }
                        }
                    }
                    Err(_) => (),
                }

                for ctx in contexts.values() {
                    ctx.run_once(Duration::ZERO);
                }

                sleeper.sleep(event_loop_timeout);
            })?;

        Ok(Context {
            sender,
            _thread: thread,
        })
    }
}

struct ConnectionFuture {
    state: Arc<Mutex<ConnectionFutureState>>,
}

struct ConnectionFutureState {
    connection_event: Option<ConnectionEvent>,
    waker: Option<Waker>,
}

impl ConnectionFuture {
    fn new() -> Self {
        ConnectionFuture {
            state: Arc::new(Mutex::new(ConnectionFutureState {
                connection_event: None,
                waker: None,
            })),
        }
    }
}

impl Future for ConnectionFuture {
    type Output = ConnectionEvent;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();

        let Some(result) = state.connection_event.take() else {
            state.waker = Some(cx.waker().clone());
            return Poll::Pending
        };
        return Poll::Ready(result);
    }
}
