use tracing::info;
use wasm_bindgen::prelude::*;

use prose_core_client::{Client as ProseClient, ClientBuilder, NoopAvatarCache};
use prose_domain::{Availability, MessageId};

use crate::cache::IndexedDBDataCache;
use crate::connector::{Connector, JSConnectionProvider};
use crate::delegate::{Delegate, JSDelegate};
use crate::types::{BareJid, FullJid, Jid, Message, MessagesArray};
use crate::util::WasmTimeProvider;

type Result<T, E = JsError> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
struct WasmError(#[from] anyhow::Error);

#[wasm_bindgen(js_name = "ProseClient")]
pub struct Client {
    client: ProseClient<IndexedDBDataCache, NoopAvatarCache>,
}

#[wasm_bindgen(js_class = "ProseClient")]
impl Client {
    pub async fn init(
        connection_provider: JSConnectionProvider,
        delegate: JSDelegate,
    ) -> Result<Client> {
        let cache = IndexedDBDataCache::new().await?;

        let client = Client {
            client: ClientBuilder::new()
                .set_connector_provider(Connector::provider(connection_provider))
                .set_data_cache(cache)
                .set_avatar_cache(NoopAvatarCache::default())
                .set_time_provider(WasmTimeProvider::default())
                .set_delegate(Some(Box::new(Delegate::new(delegate))))
                .build(),
        };

        Ok(client)
    }

    pub async fn connect(&self, jid: FullJid, password: String) -> Result<()> {
        let jid = jid::FullJid::from(jid);

        info!("Connect {} - {}", jid, password);

        self.client
            .connect(&jid, password, Availability::Available, None)
            .await?;

        Ok(())
    }

    #[wasm_bindgen(js_name = "sendMessage")]
    pub async fn send_message(&self, to: Jid, body: String) -> Result<()> {
        let to = jid::Jid::from(to);

        info!("Sending message to {}…", to);

        self.client
            .send_message(to, body)
            .await
            .map_err(WasmError::from)?;
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
        from: BareJid,
        since: Option<String>,
        load_from_server: bool,
    ) -> Result<MessagesArray, JsValue> {
        let since = since.map(|id| MessageId(id));
        let from = jid::BareJid::from(from);

        let messages = self
            .client
            .load_latest_messages(&from, since.as_ref(), load_from_server)
            .await
            .unwrap();

        Ok(messages
            .into_iter()
            .map(|m| JsValue::from(Message::from(m)))
            .collect::<js_sys::Array>()
            .unchecked_into::<MessagesArray>())
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

    // #[wasm_bindgen(js_name = "loadMessagesWithIDs")]
    // pub async fn load_messages_with_ids(
    //     &self,
    //     conversation: String,
    //     ids: Vec<JsValue>,
    // ) -> Result<JsValue, JsValue> {
    //     info!("Loading messages in conversation {}…", conversation);
    //
    //     let message_ids: Vec<MessageId> = ids
    //         .into_iter()
    //         .map(|m| JsValue::from(Message::from(m)))
    //         .collect();
    //
    //     let messages = self
    //         .client
    //         .load_messages_with_ids(&BareJid::from_str(&conversation).unwrap(), &message_ids)
    //         .await
    //         .map_err(|err| JsValue::from(err.to_string()))?;
    //
    //     info!("Found {} messages.", messages.len());
    //
    //     Ok(serde_wasm_bindgen::to_value(&messages).unwrap())
    // }
}
