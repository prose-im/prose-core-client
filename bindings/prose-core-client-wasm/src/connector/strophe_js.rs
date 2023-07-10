use std::rc::Rc;
use std::str::FromStr;

use anyhow::Result;
use async_trait::async_trait;
use jid::FullJid;
use minidom::Element;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use prose_xmpp::client::ConnectorProvider;
use prose_xmpp::connector::{
    Connection as ConnectionTrait, ConnectionError, ConnectionEvent, ConnectionEventHandler,
    Connector as ConnectorTrait,
};

#[wasm_bindgen]
extern "C" {
    type StropheJSClient;

    #[wasm_bindgen(constructor)]
    fn new() -> StropheJSClient;

    #[wasm_bindgen(method, js_name = "setEventHandler")]
    fn set_event_handler(this: &StropheJSClient, handlers: EventHandler);

    #[wasm_bindgen(method)]
    async fn connect(this: &StropheJSClient, jid: String, password: String);

    #[wasm_bindgen(method)]
    fn disconnect(this: &StropheJSClient);

    #[wasm_bindgen(method, js_name = "sendStanza")]
    fn send_stanza(this: &StropheJSClient, stanza: String);
}

pub struct Connector {}

impl Connector {
    pub fn provider() -> ConnectorProvider {
        || Box::new(Connector {})
    }
}

#[async_trait(? Send)]
impl ConnectorTrait for Connector {
    async fn connect(
        &self,
        jid: &FullJid,
        password: &str,
        event_handler: ConnectionEventHandler,
    ) -> Result<Box<dyn ConnectionTrait>, ConnectionError> {
        let client = Rc::new(StropheJSClient::new());

        let event_handler = EventHandler {
            connection: Connection {
                client: client.clone(),
            },
            handler: event_handler,
        };
        client.set_event_handler(event_handler);
        client.connect(jid.to_string(), password.to_string()).await;

        Ok(Box::new(Connection { client }))
    }
}

pub struct Connection {
    client: Rc<StropheJSClient>,
}

impl ConnectionTrait for Connection {
    fn send_stanza(&self, stanza: Element) -> Result<()> {
        self.client.send_stanza(String::from(&stanza));
        // self.client.send_stanza(DomParser::new().unwrap());
        Ok(())
    }

    fn disconnect(&self) {
        self.client.disconnect()
    }
}

#[wasm_bindgen]
pub struct EventHandler {
    connection: Connection,
    handler: ConnectionEventHandler,
}

#[wasm_bindgen]
impl EventHandler {
    pub fn handle_disconnect(&self, error: String) {
        let fut = (self.handler)(
            &self.connection,
            ConnectionEvent::Disconnected {
                error: Some(ConnectionError::Generic { msg: error }),
            },
        );
        spawn_local(async move { fut.await })
    }

    pub fn handle_timeout(&self) {
        let fut = (self.handler)(&self.connection, ConnectionEvent::TimeoutTimer);
        spawn_local(async move { fut.await })
    }

    pub fn handle_ping_timeout(&self) {
        let fut = (self.handler)(&self.connection, ConnectionEvent::PingTimer);
        spawn_local(async move { fut.await })
    }

    pub fn handle_stanza(&self, stanza: String) {
        let fut = (self.handler)(
            &self.connection,
            ConnectionEvent::Stanza(
                Element::from_str(&stanza).expect("Failed to parse received stanza"),
            ),
        );
        spawn_local(async move { fut.await })
    }
}
