// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::rc::Rc;
use std::str::FromStr;

use anyhow::Result;
use async_trait::async_trait;
use jid::FullJid;
use minidom::Element;
use secrecy::{ExposeSecret, Secret};
use thiserror::Error;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::DomException;

use prose_xmpp::client::ConnectorProvider;
use prose_xmpp::connector::{
    Connection as ConnectionTrait, ConnectionError, ConnectionEvent, ConnectionEventHandler,
    Connector as ConnectorTrait,
};

use crate::client::ClientConfig;
use crate::types::ConnectionErrorType;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export interface ProseConnectionProvider {
    provideConnection(config: ClientConfig): ProseConnection
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
    #[wasm_bindgen]
    pub type ProseConnectionProvider;

    #[wasm_bindgen(method, js_name = "provideConnection")]
    pub fn provide_connection(this: &ProseConnectionProvider, config: ClientConfig)
        -> JSConnection;
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
    fn send_stanza(this: &JSConnection, stanza: String) -> Result<(), DomException>;
}

#[wasm_bindgen(js_name = "ProseConnectionEventHandler")]
pub struct EventHandler {
    connection: Connection,
    handler: Rc<ConnectionEventHandler>,
}

pub struct Connector {
    provider: Rc<ProseConnectionProvider>,
    config: ClientConfig,
}

impl Connector {
    pub fn provider(provider: ProseConnectionProvider, config: ClientConfig) -> ConnectorProvider {
        let provider = Rc::new(provider);

        Box::new(move || {
            Box::new(Connector {
                provider: provider.clone(),
                config: config.clone(),
            })
        })
    }
}

#[async_trait(? Send)]
impl ConnectorTrait for Connector {
    async fn connect(
        &self,
        jid: &FullJid,
        password: Secret<String>,
        event_handler: ConnectionEventHandler,
    ) -> Result<Box<dyn ConnectionTrait>, ConnectionError> {
        let client = Rc::new(self.provider.provide_connection(self.config.clone()));
        let event_handler = Rc::new(event_handler);

        let event_handler = EventHandler {
            connection: Connection::new(client.clone()),
            handler: event_handler,
        };
        client.set_event_handler(event_handler);
        let result = client
            .connect(jid.to_string(), password.expose_secret().to_string())
            .await;

        if let Err(err) = result {
            let Some(code) = err.as_f64().map(|code| code as i32) else {
                return Err(ConnectionError::Generic {
                    msg: "strophe.js connector returned an invalid error code.".to_string(),
                });
            };

            let Ok(error_type) = ConnectionErrorType::try_from(code) else {
                return Err(ConnectionError::Generic {
                    msg: "strophe.js connector returned an invalid error code.".to_string(),
                });
            };

            return Err(ConnectionError::from(error_type));
        }

        Ok(Box::new(Connection { client }))
    }
}

pub struct Connection {
    client: Rc<JSConnection>,
}

impl Connection {
    fn new(client: Rc<JSConnection>) -> Self {
        Connection { client }
    }
}

impl ConnectionTrait for Connection {
    fn send_stanza(&self, stanza: Element) -> Result<()> {
        self.client
            .send_stanza(String::from(&stanza))
            .map_err(|err| JSConnectionError::from(err))?;
        Ok(())
    }

    fn disconnect(&self) {
        self.client.disconnect()
    }
}

#[wasm_bindgen(js_class = "ProseConnectionEventHandler")]
impl EventHandler {
    #[wasm_bindgen(js_name = "handleDisconnect")]
    pub fn handle_disconnect(&self, error: Option<String>) {
        let fut = (self.handler)(
            &self.connection,
            ConnectionEvent::Disconnected {
                error: error.map(|error| ConnectionError::Generic { msg: error }),
            },
        );
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

    #[wasm_bindgen(js_name = "handlePingTimer")]
    pub fn handle_ping_timer(&self) {
        let fut = (self.handler)(&self.connection, ConnectionEvent::PingTimer);
        spawn_local(async move { fut.await })
    }

    #[wasm_bindgen(js_name = "handleTimeoutTimer")]
    pub fn handle_timeout_timer(&self) {
        let fut = (self.handler)(&self.connection, ConnectionEvent::TimeoutTimer);
        spawn_local(async move { fut.await })
    }
}

#[derive(Error, Debug)]
pub enum JSConnectionError {
    #[error("DomException {name}: {message}")]
    DomException { name: String, message: String },
}

impl From<DomException> for JSConnectionError {
    fn from(value: DomException) -> Self {
        JSConnectionError::DomException {
            name: value.name(),
            message: value.message(),
        }
    }
}
