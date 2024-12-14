// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use futures::stream::StreamExt;
use futures::SinkExt;
use jid::FullJid;
use minidom::Element;
use secrecy::{ExposeSecret, SecretString};
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use tokio::{task, time};
use tokio_xmpp::starttls::ServerConfig;
use tokio_xmpp::{AsyncClient, Error, Event, Packet};
use tracing::error;

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
        ) -> Result<AsyncClient<ServerConfig>, ConnectionError> {
            let mut client = AsyncClient::new(jid.clone(), password.expose_secret());
            client.set_reconnect(false);

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

pub struct Connection {
    sender: Arc<UnboundedSender<Packet>>,
    _stream_read_handle: Option<JoinHandle<()>>,
    _stream_write_handle: Option<JoinHandle<()>>,
    _ping_handle: Option<JoinHandle<()>>,
    _timeout_handle: Option<JoinHandle<()>>,
}

impl Connection {
    fn new(client: AsyncClient<ServerConfig>, event_handler: ConnectionEventHandler) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();

        let sender = Arc::new(tx);

        let (mut writer, mut reader) = client.split();
        let event_handler = Arc::new(event_handler);

        let read_handle = {
            let sender = sender.clone();
            let event_handler = event_handler.clone();

            task::spawn(async move {
                while let Some(event) = reader.next().await {
                    let conn = Connection::new_with_sender(sender.clone());

                    match event {
                        Event::Disconnected(err) => {
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
                        Event::Online { .. } => (),
                        Event::Stanza(stanza) => {
                            #[cfg(feature = "trace-stanzas")]
                            tracing::info!(direction = "IN", "{}", String::from(&stanza));
                            (event_handler)(Box::new(conn), ConnectionEvent::Stanza(stanza)).await;
                        }
                    }
                }
            })
        };

        let write_handle = task::spawn(async move {
            while let Some(packet) = rx.recv().await {
                if let Err(err) = writer.send(packet).await {
                    error!("cannot send Stanza to internal channel: {}", err);
                    break;
                }
            }
        });

        let ping_handle = {
            let sender = sender.clone();
            let event_handler = event_handler.clone();

            task::spawn(async move {
                let mut interval = time::interval(Duration::from_secs(60));

                loop {
                    let conn = Connection::new_with_sender(sender.clone());
                    interval.tick().await;
                    let fut = (event_handler)(Box::new(conn), ConnectionEvent::PingTimer);
                    task::spawn(async move { fut.await });
                }
            })
        };

        let timeout_handle = {
            let sender = sender.clone();
            let event_handler = event_handler.clone();

            task::spawn(async move {
                let mut interval = time::interval(Duration::from_secs(2));

                loop {
                    let conn = Connection::new_with_sender(sender.clone());
                    interval.tick().await;
                    let fut = (event_handler)(Box::new(conn), ConnectionEvent::TimeoutTimer);
                    task::spawn(async move { fut.await });
                }
            })
        };

        Connection {
            sender,
            _stream_read_handle: Some(read_handle),
            _stream_write_handle: Some(write_handle),
            _ping_handle: Some(ping_handle),
            _timeout_handle: Some(timeout_handle),
        }
    }

    fn new_with_sender(sender: Arc<UnboundedSender<Packet>>) -> Self {
        Connection {
            sender,
            _stream_read_handle: None,
            _stream_write_handle: None,
            _ping_handle: None,
            _timeout_handle: None,
        }
    }
}

impl ConnectionTrait for Connection {
    fn send_stanza(&self, stanza: Element) -> Result<()> {
        #[cfg(feature = "trace-stanzas")]
        tracing::info!(direction = "OUT", "{}", String::from(&stanza));
        self.sender.send(Packet::Stanza(stanza))?;
        Ok(())
    }

    fn disconnect(&self) {
        self.sender.send(Packet::StreamEnd).unwrap()
    }
}
