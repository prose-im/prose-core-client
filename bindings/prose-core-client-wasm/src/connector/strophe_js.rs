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

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export interface ProseConnectionProvider {
    provideConnection(): ProseConnection
}

export interface ProseConnection {
    setEventHandler(handler: ProseConnectionEventHandler): void
    connect(jid: string, password: string): Promise<void>
    disconnect(): void
    sendStanza(stanza: string): void
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ProseConnectionProvider")]
    pub type JSConnectionProvider;

    #[wasm_bindgen(method, js_name = "provideConnection")]
    pub fn provide_connection(this: &JSConnectionProvider) -> JSConnection;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ProseConnection")]
    pub type JSConnection;

    #[wasm_bindgen(method, js_name = "setEventHandler")]
    fn set_event_handler(this: &JSConnection, handlers: EventHandler);

    #[wasm_bindgen(method, catch)]
    async fn connect(this: &JSConnection, jid: String, password: String) -> Result<(), JsValue>;

    #[wasm_bindgen(method)]
    fn disconnect(this: &JSConnection);

    #[wasm_bindgen(method, catch, js_name = "sendStanza")]
    fn send_stanza(this: &JSConnection, stanza: String) -> Result<(), JsValue>;
}

#[wasm_bindgen(js_name = "ProseConnectionEventHandler")]
pub struct EventHandler {
    connection: Connection,
    handler: ConnectionEventHandler,
}

pub struct Connector {
    provider: Rc<JSConnectionProvider>,
}

impl Connector {
    pub fn provider(provider: JSConnectionProvider) -> ConnectorProvider {
        let provider = Rc::new(provider);
        Box::new(move || {
            Box::new(Connector {
                provider: provider.clone(),
            })
        })
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
        let client = Rc::new(self.provider.provide_connection());

        let event_handler = EventHandler {
            connection: Connection {
                client: client.clone(),
            },
            handler: event_handler,
        };
        client.set_event_handler(event_handler);
        // TODO: Handle error
        client
            .connect(jid.to_string(), password.to_string())
            .await
            .unwrap();

        Ok(Box::new(Connection { client }))
    }
}

pub struct Connection {
    client: Rc<JSConnection>,
}

impl ConnectionTrait for Connection {
    fn send_stanza(&self, stanza: Element) -> Result<()> {
        // TODO: Handle result
        self.client.send_stanza(String::from(&stanza)).unwrap();
        // self.client.send_stanza(DomParser::new().unwrap());
        Ok(())
    }

    fn disconnect(&self) {
        self.client.disconnect()
    }
}

#[wasm_bindgen(js_class = "ProseConnectionEventHandler")]
impl EventHandler {
    #[wasm_bindgen(js_name = "handleDisconnect")]
    pub fn handle_disconnect(&self, error: String) {
        let fut = (self.handler)(
            &self.connection,
            ConnectionEvent::Disconnected {
                error: Some(ConnectionError::Generic { msg: error }),
            },
        );
        spawn_local(async move { fut.await })
    }

    #[wasm_bindgen(js_name = "handleTimeout")]
    pub fn handle_timeout(&self) {
        let fut = (self.handler)(&self.connection, ConnectionEvent::TimeoutTimer);
        spawn_local(async move { fut.await })
    }

    #[wasm_bindgen(js_name = "handlePingTimeout")]
    pub fn handle_ping_timeout(&self) {
        let fut = (self.handler)(&self.connection, ConnectionEvent::PingTimer);
        spawn_local(async move { fut.await })
    }

    #[wasm_bindgen(js_name = "handleStanza")]
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
