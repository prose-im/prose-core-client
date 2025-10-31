// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context as _, Result};
use async_trait::async_trait;
use futures::stream::StreamExt;
use jid::FullJid;
use minidom::Element;
use secrecy::{ExposeSecret, SecretString};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time;
use tokio_xmpp::Stanza;
use tokio_xmpp::{Client, Error, Event};

use crate::client::ConnectorProvider;
use crate::connector::{
    Connection as ConnectionTrait, ConnectionError, ConnectionEvent, ConnectionEventHandler,
    Connector as ConnectorTrait,
};

pub struct Connector {}

impl Connector {
    pub fn provider() -> ConnectorProvider {
        Box::new(|| Box::new(Connector {}))
    }
}

#[async_trait]
impl ConnectorTrait for Connector {
    async fn connect(
        &self,
        jid: &FullJid,
        password: SecretString,
        event_handler: ConnectionEventHandler,
    ) -> Result<Box<dyn ConnectionTrait>, ConnectionError> {
        async fn connect(
            jid: &FullJid,
            password: SecretString,
        ) -> Result<XMPPClient, ConnectionError> {
            let mut client = init_client(jid, password);

            while let Some(event) = client.next().await {
                match event {
                    Event::Disconnected(Error::Auth(_)) => {
                        return Err(ConnectionError::InvalidCredentials);
                    }
                    Event::Disconnected(e) => {
                        return Err(ConnectionError::Generic { msg: e.to_string() });
                    }
                    Event::Online { .. } => break,
                    Event::Stanza(stanza) => {
                        return Err(ConnectionError::Generic {
                            msg: format!("Received unexpected stanza {:?}", stanza),
                        });
                    }
                }
            }

            Ok(client)
        }

        connect(jid, password).await.map(|client| {
            Box::new(Connection::new(client, event_handler)) as Box<dyn ConnectionTrait>
        })
    }
}

enum ConnectionTraitEvent {
    Stanza(Stanza),
    Disconnect,
}

pub struct Connection {
    sender: Arc<mpsc::UnboundedSender<ConnectionTraitEvent>>,
    _handle: Option<JoinHandle<()>>,
}

impl Connection {
    fn new(mut client: XMPPClient, event_handler: ConnectionEventHandler) -> Self {
        let mut ping_interval = time::interval(Duration::from_secs(60));
        let mut timeout_interval = time::interval(Duration::from_secs(2));

        let (tx, mut rx) = mpsc::unbounded_channel::<ConnectionTraitEvent>();
        let sender = Arc::new(tx);

        let handle = tokio::spawn({
            let sender = Arc::clone(&sender);

            async move {
                // NOTE: Using `tokio::select!` reduces runtime overhead
                //   compared to spawning multiple concurrent tasks.
                //   Sending and receiving stanzas doesn’t happen concurrently,
                //   but if we had multiple tasks we’d have to wrap the client
                //   in a lock — causing delays and potential deadlocks which
                //   is even worse. Also we don’t really care about starving
                //   the timer tasks as it is effectively useless if we are
                //   already sending and receiving stanzas. Finally, graceful
                //   shutdowns might get a tiny bit delayed, but it’s not a
                //   problem (particularly compared to the reduced overhead).
                loop {
                    tokio::select! {
                        event = client.next() => match event {
                            Some(Event::Disconnected(err)) => {
                                let conn = Connection::new_with_sender(Arc::clone(&sender));
                                (event_handler)(
                                    Box::new(conn),
                                    ConnectionEvent::Disconnected {
                                        error: Some(ConnectionError::Generic {
                                            msg: err.to_string(),
                                        }),
                                    },
                                )
                                .await;
                                break;
                            }
                            Some(Event::Online { .. }) => (),
                            Some(Event::Stanza(stanza)) => {
                                let element = match Element::try_from(stanza) {
                                    Ok(element) => element
                                };

                                #[cfg(feature = "trace-stanzas")]
                                tracing::info!(direction = "IN", "{}", String::from(&element));

                                let conn = Connection::new_with_sender(Arc::clone(&sender));
                                (event_handler)(Box::new(conn), ConnectionEvent::Stanza(element)).await;
                            }
                            None => break
                        },

                        stanza = rx.recv() => match stanza {
                            Some(ConnectionTraitEvent::Stanza(stanza)) => {
                                if let Err(err) = client.send_stanza(stanza).await {
                                    tracing::error!("cannot send Stanza to internal channel: {err}");
                                    break;
                                }
                            },
                            Some(ConnectionTraitEvent::Disconnect) => {
                                if let Err(err) = client.send_end().await {
                                    tracing::error!("cannot send Stanza to internal channel: {err}");
                                    break;
                                }
                                break
                            },
                            None => break,
                        },

                        _ = ping_interval.tick() => {
                            let conn = Connection::new_with_sender(Arc::clone(&sender));
                            event_handler(Box::new(conn), ConnectionEvent::PingTimer).await
                        },

                        _ = timeout_interval.tick() => {
                            let conn = Connection::new_with_sender(Arc::clone(&sender));
                            event_handler(Box::new(conn), ConnectionEvent::TimeoutTimer).await
                        },
                    }
                }
            }
        });

        Connection {
            sender,
            _handle: Some(handle),
        }
    }

    fn new_with_sender(sender: Arc<mpsc::UnboundedSender<ConnectionTraitEvent>>) -> Self {
        Connection {
            sender,
            _handle: None,
        }
    }
}

impl ConnectionTrait for Connection {
    fn send_stanza(&self, stanza: Element) -> Result<()> {
        #[cfg(feature = "trace-stanzas")]
        tracing::info!(direction = "OUT", "{}", String::from(&stanza));

        let stanza = Stanza::try_from(stanza)
            .context("Could not convert `minidom::Element` to `tokio_xmpp::Stanza`")?;

        self.sender
            .send(ConnectionTraitEvent::Stanza(stanza))
            .context("Could not send stanza to `mpsc::UnboundedSender`")
    }

    fn disconnect(&self) {
        if let Err(err) = self.sender.send(ConnectionTraitEvent::Disconnect) {
            tracing::error!("Error when sending disconnect: {err}");
        };
    }
}

type XMPPClient = Client;

#[cfg(feature = "insecure-tcp")]
fn init_client(jid: &FullJid, password: SecretString) -> XMPPClient {
    Client::new_plaintext(
        jid.clone().into(),
        password.expose_secret().to_string(),
        tokio_xmpp::connect::DnsConfig::Addr {
            addr: format!("{}:5222", jid.domain()),
        },
        Timeouts::default(),
    )
}

#[cfg(not(feature = "insecure-tcp"))]
fn init_client(jid: &FullJid, password: SecretString) -> XMPPClient {
    Client::new(jid.clone(), password.expose_secret())
}
