// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use jid::FullJid;
use minidom::Element;
use parking_lot::Mutex;
use pretty_assertions::assert_eq;

use crate::tests::client::helpers::test_message_queue::MessageType;
use prose_core_client::Secret;
use prose_xmpp::client::ConnectorProvider;
use prose_xmpp::connector::{
    Connection as ConnectionTrait, ConnectionError, ConnectionEvent, ConnectionEventHandler,
    Connector as ConnectorTrait,
};

use super::{ElementExt, TestMessageQueue};

#[derive(Clone)]
pub struct Connector {
    messages: TestMessageQueue,
    current_connection: Arc<Mutex<Option<Connection>>>,
}

impl Connector {
    pub fn new(messages: TestMessageQueue) -> Self {
        Self {
            messages: messages.clone(),
            current_connection: Default::default(),
        }
    }

    pub fn provider(&self) -> ConnectorProvider {
        let connector = self.clone();
        Box::new(move || Box::new(connector.clone()))
    }

    pub async fn receive_next(&self) {
        let Some(connection) = self.current_connection.lock().clone() else {
            panic!("Tried to receive next stanza, but client is not connected.");
        };

        let Some(received_element) = connection.inner.messages.pop_receive() else {
            panic!("Tried to receive next stanza, but no stanza is queued for reception. Try to call recv! first.");
        };

        (connection.inner.event_handler)(&connection, ConnectionEvent::Stanza(received_element))
            .await;
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
        let connection = Connection {
            inner: Arc::new(ConnectionInner {
                messages: self.messages.clone(),
                event_handler,
            }),
        };
        self.current_connection.lock().replace(connection.clone());
        Ok(Box::new(connection))
    }
}

#[derive(Clone)]
struct Connection {
    inner: Arc<ConnectionInner>,
}

struct ConnectionInner {
    messages: TestMessageQueue,
    event_handler: ConnectionEventHandler,
}

impl ConnectionTrait for Connection {
    fn send_stanza(&self, sent_element: Element) -> Result<()> {
        let Some((expected_element, file, line)) = self.inner.messages.pop_send() else {
            let mut panic_message = format!(
                "Unexpected message sent:\n\n{}",
                sent_element
                    .to_pretty_printed_xml()
                    .expect("Failed to convert cached stanza to XML"),
            );

            if let Some((message, file, line)) = self.inner.messages.pop_message() {
                let element = match message {
                    MessageType::In(elem) => elem
                        .to_pretty_printed_xml()
                        .expect("Failed to convert cached stanza to XML"),
                    MessageType::Out(elem) => elem
                        .to_pretty_printed_xml()
                        .expect("Failed to convert cached stanza to XML"),
                    MessageType::Event(event) => format!("{:?}", event),
                };

                panic_message.push_str(&format!(
                    "\n\nNext expected message is:\n\n{element}\n\nScheduled at:\n{file}:{line}",
                ))
            } else {
                panic_message.push_str("\n\nThere were no further messages scheduled.")
            }

            panic!("{}", panic_message);
        };

        assert_eq!(
            expected_element
                .to_pretty_printed_xml()
                .expect("Failed to convert cached stanza to XML"),
            sent_element
                .to_pretty_printed_xml()
                .expect("Failed to convert received stanza to XML"),
            "\n\n➡️ Assertion failed at:\n{}:{}",
            file,
            line
        );

        while let Some(received_element) = self.inner.messages.pop_receive() {
            let fut = (self.inner.event_handler)(self, ConnectionEvent::Stanza(received_element));
            tokio::spawn(async move { fut.await });
        }

        Ok(())
    }

    fn disconnect(&self) {}
}
