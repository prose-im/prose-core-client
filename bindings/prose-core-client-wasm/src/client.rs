use std::str::FromStr;
use std::time::SystemTime;

use jid::{BareJid, FullJid, Jid};
use tracing::info;
use wasm_bindgen::prelude::*;

use prose_core_client::{Client as ProseClient, ClientBuilder, NoopAvatarCache, NoopDataCache};
use prose_domain::{Availability, MessageId};
use prose_xmpp::TimeProvider;

use crate::connector::{Connector, JSConnectionProvider};
use crate::delegate::{Delegate, JSDelegate};
use crate::error::JSConnectionError;
use crate::types::Message;

struct WasmTimeProvider {}

impl TimeProvider for WasmTimeProvider {
    fn now(&self) -> SystemTime {
        use js_sys::Date;
        use std::time::Duration;

        let now_ms = Date::now();
        let secs = (now_ms / 1000.0).floor() as u64;
        let nanos = (now_ms % 1000.0 * 1_000_000.0) as u32; // Convert remaining milliseconds to nanoseconds

        let duration = Duration::new(secs, nanos);
        SystemTime::UNIX_EPOCH + duration
    }
}

#[wasm_bindgen(js_name = "ProseClient")]
pub struct Client {
    client: ProseClient<NoopDataCache, NoopAvatarCache>,
}

#[wasm_bindgen(js_class = "ProseClient")]
impl Client {
    pub async fn init(
        connection_provider: JSConnectionProvider,
        delegate: JSDelegate,
    ) -> Result<Client, JsValue> {
        let client = Client {
            client: ClientBuilder::new()
                .set_connector_provider(Connector::provider(connection_provider))
                .set_data_cache(NoopDataCache::default())
                .set_avatar_cache(NoopAvatarCache::default())
                .set_time_provider(WasmTimeProvider {})
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

    #[wasm_bindgen(js_name = "sendMessage")]
    pub async fn send_message(&self, to: String, body: String) -> Result<(), JsValue> {
        info!("Sending message to {}…", to);

        let jid = Jid::from_str(&to).map_err(|err| JsValue::from(err.to_string()))?;

        self.client
            .send_message(jid, body)
            .await
            .map_err(|err| JsValue::from(err.to_string()))?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "loadContacts")]
    pub async fn load_contacts(&self) -> Result<JsValue, JsValue> {
        let contacts = self.client.load_contacts(Default::default()).await.unwrap();
        Ok(serde_wasm_bindgen::to_value(&contacts).unwrap())
    }

    #[wasm_bindgen(js_name = "loadLatestMessages")]
    pub async fn load_latest_messages(
        &self,
        from: String,
        since: Option<String>,
        load_from_server: bool,
    ) -> Result<JsValue, JsValue> {
        let from = BareJid::from_str(&from).unwrap();
        let since = since.map(|id| MessageId(id));

        let messages = self
            .client
            .load_latest_messages(&from, since.as_ref(), load_from_server)
            .await
            .unwrap();

        let messages: Vec<Message> = messages.into_iter().map(Into::into).collect();

        Ok(serde_wasm_bindgen::to_value(&messages).unwrap())
    }

    // #[wasm_bindgen(js_name = "loadMessagesBefore")]
    // pub async fn load_messages_before(
    //     &self,
    //     from: BareJid,
    //     before: MessageId,
    // ) -> Result<MessagesPage, ClientError> {
    //     let page = self.client.load_messages_before(&from, &before).await?;
    //     Ok(page.into())
    // }

    #[wasm_bindgen(js_name = "loadMessagesWithIDs")]
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
