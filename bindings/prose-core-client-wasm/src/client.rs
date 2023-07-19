use anyhow::anyhow;
use tracing::info;
use wasm_bindgen::prelude::*;

use prose_core_client::avatar_cache::NoopAvatarCache;
use prose_core_client::data_cache::indexed_db::IndexedDBDataCache;
use prose_core_client::{Client as ProseClient, ClientBuilder};
use prose_domain::{Availability, MessageId};

use crate::connector::{Connector, JSConnectionProvider};
use crate::delegate::{Delegate, JSDelegate};
use crate::types::{BareJid, FullJid, Jid, MessagesArray};
use crate::util::WasmTimeProvider;

type Result<T, E = JsError> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct WasmError(#[from] anyhow::Error);

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
    pub async fn send_message(&self, to: &Jid, body: String) -> Result<()> {
        let to = jid::Jid::from(to.clone());

        info!("Sending message to {}…", to);

        self.client
            .send_message(to, body)
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "loadContacts")]
    pub async fn load_contacts(&self) -> Result<JsValue, JsError> {
        let contacts = self
            .client
            .load_contacts(Default::default())
            .await
            .map_err(WasmError::from)?;
        Ok(serde_wasm_bindgen::to_value(&contacts)?)
    }

    #[wasm_bindgen(js_name = "loadLatestMessages")]
    pub async fn load_latest_messages(
        &self,
        from: &BareJid,
        since: Option<String>,
        load_from_server: bool,
    ) -> Result<MessagesArray, JsError> {
        let since = since.map(|id| MessageId(id));
        let from = jid::BareJid::from(from.clone());

        let messages = self
            .client
            .load_latest_messages(&from, since.as_ref(), load_from_server)
            .await
            .map_err(WasmError::from)?;

        Ok(messages.into())
    }

    #[wasm_bindgen(js_name = "loadMessagesWithIDs")]
    pub async fn load_messages_with_ids(
        &self,
        conversation: &BareJid,
        message_ids: &StringArray,
    ) -> Result<MessagesArray, JsError> {
        info!("Loading messages in conversation {:?}…", conversation);

        let message_ids: Vec<MessageId> = Vec::<String>::try_from(message_ids)?
            .into_iter()
            .map(|id| MessageId(id))
            .collect();

        let messages = self
            .client
            .load_messages_with_ids(&(conversation.clone().into()), message_ids.as_slice())
            .await
            .map_err(WasmError::from)?;

        info!("Found {} messages.", messages.len());

        Ok(messages.into())
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
}

// To have a correct typing annotation generated for TypeScript, declare a custom type.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "string[]")]
    pub type StringArray;
}

impl TryFrom<&StringArray> for Vec<String> {
    type Error = WasmError;

    fn try_from(value: &StringArray) -> std::result::Result<Self, Self::Error> {
        let js_val: &JsValue = value.as_ref();
        let array: &js_sys::Array = js_val
            .dyn_ref()
            .ok_or_else(|| WasmError(anyhow!("The argument must be an array")))?;

        let length: usize = array
            .length()
            .try_into()
            .map_err(|err| WasmError(anyhow!("Failed to determine array length. {}", err)))?;

        let mut typed_array = Vec::<String>::with_capacity(length);
        for js in array.iter() {
            let elem = js
                .as_string()
                .ok_or(WasmError(anyhow!("Couldn't unwrap String from Array")))?;
            typed_array.push(elem);
        }

        Ok(typed_array)
    }
}
