// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use jid::FullJid;
use minidom::Element;
use pretty_assertions::assert_eq;
use prose_core_client::Secret;

use prose_xmpp::client::ConnectorProvider;
use prose_xmpp::connector::{
    Connection as ConnectionTrait, ConnectionError, ConnectionEvent, ConnectionEventHandler,
    Connector as ConnectorTrait,
};

use crate::tests::helpers::test_message_queue::TestMessageQueue;

use super::element_ext::ElementExt;

#[derive(Clone)]
pub struct Connector {
    messages: TestMessageQueue,
}

impl Connector {
    pub fn provider(messages: TestMessageQueue) -> ConnectorProvider {
        Box::new(move || {
            Box::new(Connector {
                messages: messages.clone(),
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
        Ok(Box::new(Connection {
            inner: Arc::new(ConnectionInner {
                messages: self.messages.clone(),
                event_handler,
            }),
        }))
    }
}

struct Connection {
    inner: Arc<ConnectionInner>,
}

struct ConnectionInner {
    messages: TestMessageQueue,
    event_handler: ConnectionEventHandler,
}

impl ConnectionTrait for Connection {
    fn send_stanza(&self, sent_element: Element) -> Result<()> {
        let Some(expected_element) = self.inner.messages.pop_send() else {
            panic!("Unexpected message sent: \n{}", String::from(&sent_element));
        };

        assert_eq!(
            expected_element
                .to_pretty_printed_xml()
                .expect("Failed to convert cached stanza to XML"),
            sent_element
                .to_pretty_printed_xml()
                .expect("Failed to convert received stanza to XML")
        );

        while let Some(received_element) = self.inner.messages.pop_receive() {
            let conn = Connection {
                inner: self.inner.clone(),
            };
            let fut = (self.inner.event_handler)(&conn, ConnectionEvent::Stanza(received_element));
            tokio::spawn(async move { fut.await });
        }

        Ok(())
    }

    fn disconnect(&self) {}
}
