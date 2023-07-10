use std::str::FromStr;

use jid::{BareJid, FullJid, Jid};
use tracing::info;
use wasm_bindgen::prelude::*;

use prose_core_client::{Client as ProseClient, ClientBuilder, NoopAvatarCache, NoopDataCache};
use prose_domain::{Availability, MessageId};

use crate::connector::Connector;
use crate::delegate::{Delegate, JSClientDelegate};
use crate::error::JSConnectionError;

// The Rust XMPPClient which interacts with the server through JSXMPPClient
#[wasm_bindgen]
pub struct RustXMPPClient {
    client: ProseClient<NoopDataCache, NoopAvatarCache>,
}

#[wasm_bindgen]
impl RustXMPPClient {
    pub async fn init(delegate: JSClientDelegate) -> Result<RustXMPPClient, JsValue> {
        let client = RustXMPPClient {
            client: ClientBuilder::new()
                .set_connector_provider(Connector::provider())
                .set_data_cache(NoopDataCache::default())
                .set_avatar_cache(NoopAvatarCache::default())
                .set_delegate(Some(Box::new(Delegate::new(delegate))))
                .build(),
        };

        Ok(client)
    }

    pub async fn connect(&self, jid: String, password: String) -> Result<(), JSConnectionError> {
        info!("Connect {} - {}", jid, password);

        let jid =
            FullJid::from_str(&format!("{}/wasm", jid)).map_err(Into::<JSConnectionError>::into)?;

        self.client
            .connect(&jid, password, Availability::Available, None)
            .await
            .map_err(Into::<JSConnectionError>::into)?;

        Ok(())
    }

    pub async fn send_message(&self, to: String, body: String) -> Result<(), JsValue> {
        info!("Sending message to {}…", to);

        let jid = Jid::from_str(&to).map_err(|err| JsValue::from(err.to_string()))?;

        self.client
            .send_message(jid, body)
            .await
            .map_err(|err| JsValue::from(err.to_string()))?;
        Ok(())
    }

    pub async fn load_messages_with_ids(
        &self,
        conversation: String,
        ids: Vec<JsValue>,
    ) -> Result<JsValue, JsValue> {
        info!("Loading messages in conversation {}…", conversation);

        let message_ids: Vec<MessageId> = ids
            .into_iter()
            .map(|v| MessageId(v.as_string().unwrap()))
            .collect();

        let messages = self
            .client
            .load_messages_with_ids(&BareJid::from_str(&conversation).unwrap(), &message_ids)
            .await
            .map_err(|err| JsValue::from(err.to_string()))?;

        info!("Found {} messages.", messages.len());

        Ok(serde_wasm_bindgen::to_value(&messages).unwrap())
    }
}
