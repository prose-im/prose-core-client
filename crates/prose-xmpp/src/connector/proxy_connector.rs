// prose-core-client/prose-xmpp
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use jid::FullJid;
use minidom::Element;
use secrecy::Secret;

use prose_wasm_utils::{spawn, SendUnlessWasm, SyncUnlessWasm};

use crate::connector::{ConnectionEvent, ConnectionEventHandler};
use crate::{Connection, ConnectionError, Connector};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait ProxyTransformer: SendUnlessWasm + SyncUnlessWasm {
    async fn send_stanza(&self, connection: &dyn Connection, stanza: Element) -> Result<()>;
    async fn receive_stanza(
        &self,
        connection: Box<dyn Connection>,
        event: ConnectionEvent,
        handler: &ConnectionEventHandler,
    );
}

pub struct ProxyConnector<C, T> {
    connector: C,
    transformer: Arc<T>,
}

impl<C: Connector, T: ProxyTransformer + SendUnlessWasm + 'static> ProxyConnector<C, T> {
    pub fn new(connector: C, transformer: T) -> Self {
        Self {
            connector,
            transformer: Arc::new(transformer),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<C: Connector, T: ProxyTransformer + SendUnlessWasm + 'static> Connector
    for ProxyConnector<C, T>
{
    async fn connect(
        &self,
        jid: &FullJid,
        password: Secret<String>,
        event_handler: ConnectionEventHandler,
    ) -> Result<Box<dyn Connection>, ConnectionError> {
        let orig_event_handler = Arc::new(event_handler);

        let event_handler: ConnectionEventHandler = Box::new({
            let transformer = self.transformer.clone();
            let orig_event_handler = orig_event_handler.clone();

            move |conn: Box<dyn Connection>, event| {
                let transformer = transformer.clone();
                let orig_event_handler = orig_event_handler.clone();

                Box::pin(async move {
                    transformer
                        .receive_stanza(conn, event, &orig_event_handler)
                        .await;
                })
            }
        });

        let connection = self.connector.connect(jid, password, event_handler).await?;

        Ok(Box::new(ProxyConnection {
            connection: connection.into(),
            transformer: self.transformer.clone(),
        }))
    }
}

struct ProxyConnection<T> {
    connection: Arc<dyn Connection>,
    transformer: Arc<T>,
}

impl<T: ProxyTransformer + 'static> Connection for ProxyConnection<T> {
    fn send_stanza(&self, stanza: Element) -> Result<()> {
        spawn({
            let transformer = self.transformer.clone();
            let connection = self.connection.clone();

            async move { transformer.send_stanza(&*connection, stanza).await }
        });
        Ok(())
    }

    fn disconnect(&self) {
        self.connection.disconnect()
    }
}
