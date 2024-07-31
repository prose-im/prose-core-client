// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Range;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use minidom::Element;
use prose_wasm_utils::sleep;
use prose_xmpp::connector::{ConnectionEvent, ConnectionEventHandler, ProxyTransformer};
use prose_xmpp::Connection;
use rand::Rng;

pub struct RandomDelayProxyTransformer(Range<u64>);

impl RandomDelayProxyTransformer {
    pub fn new(range: Range<u64>) -> Self {
        Self(range)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl ProxyTransformer for RandomDelayProxyTransformer {
    async fn send_stanza(&self, connection: &dyn Connection, stanza: Element) -> Result<()> {
        let value = { rand::thread_rng().gen_range(self.0.clone()) };
        sleep(Duration::from_millis(value)).await;
        connection.send_stanza(stanza)
    }

    async fn receive_stanza(
        &self,
        connection: Box<dyn Connection>,
        event: ConnectionEvent,
        handler: &ConnectionEventHandler,
    ) {
        (handler)(connection, event).await;
    }
}
