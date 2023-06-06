use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use libstrophe::Stanza;

use crate::connector::{ConnectionConfiguration, Connector};
use crate::{Connection, ConnectionError, ConnectionEvent};

pub struct TestConnector {
    connection: Arc<TestConnection>,
}

#[async_trait]
impl Connector for TestConnector {
    async fn connect(
        &self,
        config: ConnectionConfiguration,
    ) -> anyhow::Result<Box<dyn Connection>, ConnectionError> {
        let mut config = config;
        (config.connection_handler)(&self.connection, &ConnectionEvent::Connect);
        *self.connection.config.lock().unwrap() = Some(config);
        Ok(Box::new(self.connection.clone()))
    }
}

#[derive(Debug, Clone)]
pub enum Response {
    Stanza(Stanza),
}

pub type SentStanzaHandler = dyn FnMut(&Stanza) -> Vec<Stanza> + Send;

pub struct TestConnection {
    config: Mutex<Option<ConnectionConfiguration>>,
    sent_stanzas: Mutex<Vec<Stanza>>,
    stanza_handler: Mutex<Option<Box<SentStanzaHandler>>>,
}

impl TestConnection {
    pub fn new() -> Arc<Self> {
        Arc::new(TestConnection {
            config: Mutex::new(None),
            sent_stanzas: Mutex::new(Vec::new()),
            stanza_handler: Mutex::new(None),
        })
    }

    pub fn set_stanza_handler<F>(&self, handler: F)
    where
        F: FnMut(&Stanza) -> Vec<Stanza> + Send + 'static,
    {
        *self.stanza_handler.lock().unwrap() = Some(Box::new(handler))
    }

    pub fn sent_stanzas(&self) -> Vec<Stanza> {
        self.sent_stanzas.lock().unwrap().clone()
    }

    pub fn sent_stanza_strings(&self) -> Vec<String> {
        self.sent_stanzas
            .lock()
            .unwrap()
            .iter()
            .map(|stanza| stanza.to_string())
            .collect()
    }

    pub fn connector(self: &Arc<Self>) -> Box<dyn Connector> {
        Box::new(TestConnector {
            connection: self.clone(),
        })
    }

    pub fn reset(&self) {
        self.sent_stanzas.lock().unwrap().clear()
    }
}

impl Connection for TestConnection {
    fn disconnect(&self) {}

    fn send_stanza(&self, stanza: Stanza) {
        let mut guard = self.stanza_handler.lock().unwrap();
        if let Some(handler) = guard.as_mut() {
            let responses = (handler)(&stanza);
            let mut lock = self.config.lock().unwrap();

            if let Some(config) = lock.as_mut() {
                for response in responses {
                    (config.stanza_handler)(self, &response)
                }
            };
        }
        self.sent_stanzas.lock().unwrap().push(stanza)
    }
}

impl Connection for Arc<TestConnection> {
    fn disconnect(&self) {
        self.as_ref().disconnect()
    }

    fn send_stanza(&self, stanza: Stanza) {
        self.as_ref().send_stanza(stanza)
    }
}
