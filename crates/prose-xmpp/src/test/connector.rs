// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use jid::FullJid;
use minidom::Element;
use parking_lot::{Mutex, RwLock};
use secrecy::Secret;
use xmpp_parsers::disco::DiscoItemsResult;
use xmpp_parsers::iq::Iq;

use crate::client::ConnectorProvider;
use crate::connector::{
    Connection as ConnectionTrait, ConnectionError, ConnectionEvent, ConnectionEventHandler,
    Connector as ConnectorTrait,
};

pub struct Connector {
    connection: Connection,
}

impl Connector {
    pub fn provider(connection: Connection) -> ConnectorProvider {
        Box::new(move || {
            Box::new(Connector {
                connection: connection.clone(),
            })
        })
    }
}

#[async_trait]
impl ConnectorTrait for Connector {
    async fn connect(
        &self,
        _jid: &FullJid,
        _password: Secret<String>,
        event_handler: ConnectionEventHandler,
    ) -> Result<Box<dyn ConnectionTrait>, ConnectionError> {
        *self.connection.inner.event_handler.write() = Some(event_handler);
        Ok(Box::new(self.connection.clone()))
    }
}

pub type SentStanzaHandler = dyn FnMut(&Element) -> Vec<Element> + Send;

#[derive(Default, Clone)]
pub struct Connection {
    inner: Arc<ConnectionInner>,
}

#[derive(Default)]
struct ConnectionInner {
    sent_stanzas: Mutex<Vec<Element>>,
    stanza_handler: Mutex<Option<Box<SentStanzaHandler>>>,
    event_handler: RwLock<Option<ConnectionEventHandler>>,
}

impl Connection {
    pub fn set_stanza_handler<F>(&self, handler: F)
    where
        F: FnMut(&Element) -> Vec<Element> + Send + 'static,
    {
        *self.inner.stanza_handler.lock() = Some(Box::new(handler))
    }

    pub fn use_start_sequence_handler(&self) {
        self.set_stanza_handler(|st| {
            if st.name() != "iq" || st.attr("id") != Some("id-2") {
                return vec![];
            }

            vec![Iq::from_result(
                "id-2",
                Some(DiscoItemsResult {
                    node: None,
                    items: vec![],
                    rsm: None,
                }),
            )
            .into()]
        });
    }

    pub fn sent_stanzas(&self) -> Vec<Element> {
        self.inner.sent_stanzas.lock().clone()
    }

    pub fn sent_stanza_strings(&self) -> Vec<String> {
        self.inner
            .sent_stanzas
            .lock()
            .iter()
            .map(String::from)
            .collect()
    }

    pub fn connector(&self) -> Box<dyn ConnectorTrait> {
        Box::new(Connector {
            connection: self.clone(),
        })
    }

    pub fn reset(&self) {
        self.inner.sent_stanzas.lock().clear()
    }

    pub async fn receive_stanza(&self, stanza: impl Into<Element>) {
        let guard = self.inner.event_handler.read();
        let event_handler = guard.as_ref().expect("No event handler registered");
        let conn = Connection {
            inner: self.inner.clone(),
        };
        (event_handler)(&conn, ConnectionEvent::Stanza(stanza.into())).await
    }
}

impl ConnectionTrait for Connection {
    fn send_stanza(&self, stanza: Element) -> Result<()> {
        let responses = if let Some(handler) = self.inner.stanza_handler.lock().as_mut() {
            (handler)(&stanza)
        } else {
            vec![]
        };

        if let Some(event_handler) = &*self.inner.event_handler.read() {
            for response in responses {
                let conn = Connection {
                    inner: self.inner.clone(),
                };
                let fut = (event_handler)(&conn, ConnectionEvent::Stanza(response));

                tokio::spawn(async move { fut.await });
            }
        }

        self.inner.sent_stanzas.lock().push(stanza);
        Ok(())
    }

    fn disconnect(&self) {}
}
