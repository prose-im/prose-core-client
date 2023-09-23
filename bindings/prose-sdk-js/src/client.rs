// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::connector::{Connector, ProseConnectionProvider};
use crate::delegate::{Delegate, JSDelegate};
use crate::types::{
    Availability, BareJid, BareJidArray, Channel, ChannelsArray, Contact, ContactsArray,
    IntoJSArray, RoomsArray, UserMetadata, UserProfile,
};
use base64::{engine::general_purpose, Engine as _};
use jid::ResourcePart;
use prose_core_client::data_cache::indexed_db::IndexedDBDataCache;
use prose_core_client::types::{SoftwareVersion, UserActivity};
use prose_core_client::{CachePolicy, Client as ProseClient, ClientBuilder};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

type Result<T, E = JsError> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct WasmError(#[from] anyhow::Error);

#[derive(Debug, PartialEq, Clone)]
#[wasm_bindgen(js_name = "ProseClientConfig")]
pub struct ClientConfig {
    /// Defines the frequency in which Pings are sent (in seconds). Useful for debugging
    /// disconnect/reconnect scenarios. Default is 60s.
    #[wasm_bindgen(js_name = "pingInterval")]
    pub ping_interval: u32,

    /// Defines if received stanzas should be logged to the console.
    #[wasm_bindgen(js_name = "logReceivedStanzas")]
    pub log_received_stanzas: bool,

    /// Defines if sent stanzas should be logged to the console.
    #[wasm_bindgen(js_name = "logSentStanzas")]
    pub log_sent_stanzas: bool,

    #[wasm_bindgen(skip)]
    pub client_name: String,

    #[wasm_bindgen(skip)]
    pub client_version: String,

    #[wasm_bindgen(skip)]
    pub client_os: Option<String>,
}

#[wasm_bindgen(js_class = "ProseClientConfig")]
impl ClientConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

    #[wasm_bindgen(getter, js_name = "clientName")]
    pub fn client_name(&self) -> String {
        self.client_name.clone()
    }

    #[wasm_bindgen(setter, js_name = "clientName")]
    pub fn set_client_name(&mut self, client_name: String) {
        self.client_name = client_name.clone()
    }

    #[wasm_bindgen(getter, js_name = "clientVersion")]
    pub fn client_version(&self) -> String {
        self.client_version.clone()
    }

    #[wasm_bindgen(setter, js_name = "clientVersion")]
    pub fn set_client_version(&mut self, client_version: String) {
        self.client_version = client_version.clone()
    }

    #[wasm_bindgen(getter, js_name = "clientOS")]
    pub fn client_os(&self) -> Option<String> {
        self.client_os.clone()
    }

    #[wasm_bindgen(setter, js_name = "clientOS")]
    pub fn set_client_os(&mut self, client_os: Option<String>) {
        self.client_os = client_os.clone()
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            ping_interval: 60,
            log_received_stanzas: false,
            log_sent_stanzas: false,
            client_name: env!("CARGO_PKG_NAME").to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            client_os: None,
        }
    }
}

#[wasm_bindgen(js_name = "ProseClient")]
pub struct Client {
    client: ProseClient<Rc<IndexedDBDataCache>, Rc<IndexedDBDataCache>>,
}

#[wasm_bindgen(js_class = "ProseClient")]
impl Client {
    pub async fn init(
        connection_provider: ProseConnectionProvider,
        delegate: JSDelegate,
        config: Option<ClientConfig>,
    ) -> Result<Client> {
        let cache = Rc::new(IndexedDBDataCache::new().await?);
        let config = config.unwrap_or_default();

        let software_version = SoftwareVersion {
            name: config.client_name.clone(),
            version: config.client_version.clone(),
            os: config.client_os.clone(),
        };

        let client = Client {
            client: ClientBuilder::new()
                .set_connector_provider(Connector::provider(connection_provider, config))
                .set_data_cache(cache.clone())
                .set_avatar_cache(cache)
                .set_delegate(Some(Box::new(Delegate::new(delegate))))
                .set_software_version(software_version)
                .build(),
        };

        Ok(client)
    }

    pub async fn connect(
        &self,
        jid: &BareJid,
        password: &str,
        availability: Availability,
    ) -> Result<()> {
        // TODO: Generate and store resource.
        let jid = jid.to_full_jid_with_resource(&ResourcePart::new("web").unwrap());

        self.client
            .connect(&jid, password, availability.into())
            .await?;

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
        self.client.disconnect().await;
        Ok(())
    }

    #[wasm_bindgen(js_name = "startObservingRooms")]
    pub async fn start_observing_rooms(&self) -> Result<()> {
        self.client
            .start_observing_rooms()
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "connectedRooms")]
    pub fn connected_rooms(&self) -> Result<RoomsArray> {
        Ok(self.client.connected_rooms().into())
    }

    #[wasm_bindgen(js_name = "loadPublicChannels")]
    pub async fn load_public_channels(&self) -> Result<ChannelsArray> {
        Ok(self
            .client
            .load_public_rooms()
            .await
            .map_err(WasmError::from)?
            .into_iter()
            .map(Channel::from)
            .collect_into_js_array::<ChannelsArray>())
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
        conversation: &BareJid,
    ) -> Result<BareJidArray> {
        let user_jids = self
            .client
            .load_composing_users(conversation.as_ref())
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

    /// XEP-0084: User Avatar
    /// https://xmpp.org/extensions/xep-0084.html
    #[wasm_bindgen(js_name = "loadAvatarDataURL")]
    pub async fn load_avatar_data_url(&self, jid: &BareJid) -> Result<Option<String>> {
        let avatar = self
            .client
            .load_avatar(jid, CachePolicy::ReturnCacheDataDontLoad)
            .await
            .map_err(WasmError::from)?;
        Ok(avatar)
    }

    /// XEP-0084: User Avatar
    /// https://xmpp.org/extensions/xep-0084.html
    #[wasm_bindgen(js_name = "saveAvatar")]
    pub async fn save_avatar(&self, image_data: &str, mime_type: &str) -> Result<()> {
        // Somehow converting the String from FileReader.readAsBinaryString via String::as_bytes()
        // did not work. Maybe just the the Blob (e.g. via gloo-file/Blob)?
        let image_data = general_purpose::STANDARD
            .decode(image_data)
            .map_err(|err| WasmError::from(anyhow::Error::from(err)))?;

        self.client
            .save_avatar(&image_data, None, None, mime_type)
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    /// XEP-0292: vCard4 Over XMPP
    /// https://xmpp.org/extensions/xep-0292.html
    #[wasm_bindgen(js_name = "loadUserProfile")]
    pub async fn load_user_profile(&self, jid: &BareJid) -> Result<Option<UserProfile>> {
        let profile = self
            .client
            .load_user_profile(jid, CachePolicy::ReturnCacheDataElseLoad)
            .await
            .map_err(WasmError::from)?;

        Ok(profile.map(Into::into))
    }

    /// XEP-0292: vCard4 Over XMPP
    /// https://xmpp.org/extensions/xep-0292.html
    #[wasm_bindgen(js_name = "saveUserProfile")]
    pub async fn save_user_profile(&self, profile: &UserProfile) -> Result<()> {
        self.client
            .save_profile((profile.clone()).into())
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "deleteCachedData")]
    pub async fn delete_cached_data(&self) -> Result<()> {
        self.client
            .delete_cached_data()
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "loadUserMetadata")]
    pub async fn load_user_metadata(&self, jid: &BareJid) -> Result<UserMetadata> {
        let metadata = self
            .client
            .load_user_metadata(jid.as_ref())
            .await
            .map_err(WasmError::from)?;
        Ok(metadata.into())
    }

    /// XMPP: Instant Messaging and Presence
    /// https://xmpp.org/rfcs/rfc6121.html#presence
    #[wasm_bindgen(js_name = "setAvailability")]
    pub async fn set_availability(&self, availability: Availability) -> Result<()> {
        self.client
            .set_availability(availability.into())
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }
}

impl From<ProseClient<Rc<IndexedDBDataCache>, Rc<IndexedDBDataCache>>> for Client {
    fn from(client: ProseClient<Rc<IndexedDBDataCache>, Rc<IndexedDBDataCache>>) -> Self {
        Client { client }
    }
}
