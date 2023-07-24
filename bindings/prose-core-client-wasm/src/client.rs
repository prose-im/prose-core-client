use microtype::Microtype;
use tracing::info;
use wasm_bindgen::prelude::*;

use prose_core_client::avatar_cache::NoopAvatarCache;
use prose_core_client::data_cache::indexed_db::IndexedDBDataCache;
use prose_core_client::types::{Availability, UserActivity};
use prose_core_client::{Client as ProseClient, ClientBuilder};
use prose_domain::{Emoji, MessageId};

use crate::connector::{Connector, JSConnectionProvider};
use crate::delegate::{Delegate, JSDelegate};
use crate::types::{
    BareJid, BareJidArray, Contact, ContactsArray, FullJid, IntoJSArray, Jid, MessagesArray,
    StringArray,
};
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

    pub async fn connect(&self, jid: &FullJid, password: &str) -> Result<()> {
        let jid = jid::FullJid::from(jid.clone());

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

    /// XEP-0308: Last Message Correction
    /// https://xmpp.org/extensions/xep-0308.html
    #[wasm_bindgen(js_name = "updateMessage")]
    pub async fn update_message(
        &self,
        conversation: &Jid,
        message_id: &str,
        body: String,
    ) -> Result<()> {
        self.client
            .update_message(
                jid::Jid::from(conversation.clone()),
                message_id.into(),
                body,
            )
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    /// XEP-0424: Message Retraction
    /// https://xmpp.org/extensions/xep-0424.html
    #[wasm_bindgen(js_name = "retractMessage")]
    pub async fn retract_message(&self, conversation: &Jid, message_id: &str) -> Result<()> {
        self.client
            .retract_message(jid::Jid::from(conversation.clone()), message_id.into())
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    /// XEP-0085: Chat State Notifications
    /// https://xmpp.org/extensions/xep-0085.html
    #[wasm_bindgen(js_name = "setUserIsComposing")]
    pub async fn set_user_is_composing(
        &self,
        conversation: &Jid,
        is_composing: bool,
    ) -> Result<()> {
        self.client
            .set_user_is_composing(jid::Jid::from(conversation.clone()), is_composing)
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    /// XEP-0108: User Activity
    /// https://xmpp.org/extensions/xep-0108.html
    #[wasm_bindgen(js_name = "sendActivity")]
    pub async fn set_user_activity(
        &self,
        icon: Option<String>,
        text: Option<String>,
    ) -> Result<()> {
        let user_activity = if let Some(icon) = &icon {
            Some(UserActivity {
                emoji: icon.clone(),
                status: text.clone(),
            })
        } else {
            None
        };

        self.client
            .set_user_activity(user_activity)
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "loadComposingUsersInConversation")]
    pub async fn load_composing_users_in_conversation(
        &self,
        conversation: &Jid,
    ) -> Result<BareJidArray> {
        let user_jids = self
            .client
            .load_composing_users(&jid::BareJid::from(conversation.bare()))
            .await
            .map_err(WasmError::from)?
            .into_iter()
            .map(|jid| BareJid::from(jid))
            .collect_into_js_array::<BareJidArray>();
        Ok(user_jids)
    }

    #[wasm_bindgen(js_name = "loadContacts")]
    pub async fn load_contacts(&self) -> Result<ContactsArray> {
        Ok(self
            .client
            .load_contacts(Default::default())
            .await
            .map_err(WasmError::from)?
            .into_iter()
            .map(|c| JsValue::from(Contact::from(c)))
            .collect_into_js_array::<ContactsArray>())
    }

    #[wasm_bindgen(js_name = "loadLatestMessages")]
    pub async fn load_latest_messages(
        &self,
        from: &BareJid,
        since: Option<String>,
        load_from_server: bool,
    ) -> Result<MessagesArray> {
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
    ) -> Result<MessagesArray> {
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

    /// XEP-0444: Message Reactions
    /// https://xmpp.org/extensions/xep-0444.html
    #[wasm_bindgen(js_name = "toggleReactionToMessage")]
    pub async fn toggle_reaction_to_message(
        &self,
        conversation: &BareJid,
        id: &str,
        emoji: &str,
    ) -> Result<()> {
        self.client
            .toggle_reaction_to_message(
                jid::BareJid::from(conversation.clone()),
                MessageId::new(id.into()),
                Emoji::new(emoji.into()),
            )
            .await
            .map_err(WasmError::from)?;
        Ok(())
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
