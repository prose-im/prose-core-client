// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use base64::{engine::general_purpose, Engine as _};
use js_sys::Array;
use tracing::{info, Level};
use tracing_subscriber::fmt::format::{FmtSpan, Pretty};
use tracing_subscriber::prelude::*;
use wasm_bindgen::prelude::*;

use prose_core_client::dtos::{RoomId, SoftwareVersion, UserStatus};
use prose_core_client::{open_store, Client as ProseClient, IndexedDBDriver, StoreAvatarCache};

use crate::connector::{Connector, ProseConnectionProvider};
use crate::delegate::{Delegate, JSDelegate};
use crate::types::{
    try_user_id_vec_from_string_array, Availability, BareJid, Channel, ChannelsArray, Contact,
    ContactsArray, IntoJSArray, SidebarItem, SidebarItemsArray, UserMetadata, UserProfile,
};

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

    #[wasm_bindgen(js_name = "loggingEnabled")]
    pub logging_enabled: bool,

    #[wasm_bindgen(skip)]
    pub logging_min_level: String,

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

    #[wasm_bindgen(getter, js_name = "loggingMinLevel")]
    pub fn logging_min_level(&self) -> String {
        self.logging_min_level.clone()
    }

    #[wasm_bindgen(setter, js_name = "loggingMinLevel")]
    pub fn set_logging_min_level(&mut self, level: String) {
        self.logging_min_level = level.clone()
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            ping_interval: 60,
            log_received_stanzas: false,
            log_sent_stanzas: false,
            logging_enabled: true,
            logging_min_level: "trace".to_string(),
            client_name: env!("CARGO_PKG_NAME").to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            client_os: None,
        }
    }
}

#[wasm_bindgen(js_name = "ProseClient")]
pub struct Client {
    client: ProseClient,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[wasm_bindgen(js_class = "ProseClient")]
impl Client {
    pub async fn init(
        connection_provider: ProseConnectionProvider,
        delegate: JSDelegate,
        config: Option<ClientConfig>,
    ) -> Result<Client> {
        let store = open_store(IndexedDBDriver::new("ProseCache2")).await?;
        let config = config.unwrap_or_default();

        if config.logging_enabled {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .without_time()
                .with_writer(
                    tracing_web::MakeWebConsoleWriter::new()
                        .with_pretty_level()
                        .with_max_level(config.logging_min_level.parse().unwrap_or(Level::TRACE)),
                )
                .with_level(false)
                .with_span_events(FmtSpan::ACTIVE);
            let perf_layer =
                tracing_web::performance_layer().with_details_from_fields(Pretty::default());

            tracing_subscriber::registry()
                .with(fmt_layer)
                .with(perf_layer)
                .init();

            info!("prose-sdk-js Version {VERSION}");
        }

        let software_version = SoftwareVersion {
            name: config.client_name.clone(),
            version: config.client_version.clone(),
            os: config.client_os.clone(),
        };

        let client = Client {
            client: ProseClient::builder()
                .set_connector_provider(Connector::provider(connection_provider, config))
                .set_store(store.clone())
                .set_avatar_cache(StoreAvatarCache::new(store))
                .set_delegate(Some(Box::new(Delegate::new(delegate))))
                .set_software_version(software_version)
                .build(),
        };

        Ok(client)
    }

    pub async fn connect(&self, jid: &BareJid, password: &str) -> Result<()> {
        self.client.connect(&jid.into(), password).await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
        self.client.disconnect().await;
        Ok(())
    }

    #[wasm_bindgen(js_name = "startObservingRooms")]
    pub async fn start_observing_rooms(&self) -> Result<()> {
        self.client
            .rooms
            .start_observing_rooms()
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "sidebarItems")]
    pub async fn sidebar_items(&self) -> SidebarItemsArray {
        self.client
            .sidebar
            .sidebar_items()
            .await
            .into_iter()
            .map(|item| {
                JsValue::from(SidebarItem {
                    dto: item,
                    client: self.client.clone(),
                })
            })
            .collect_into_js_array::<SidebarItemsArray>()
    }

    #[wasm_bindgen(js_name = "loadPublicChannels")]
    pub async fn load_public_channels(&self) -> Result<ChannelsArray> {
        Ok(self
            .client
            .rooms
            .load_public_rooms()
            .await
            .map_err(WasmError::from)?
            .into_iter()
            .map(Channel::from)
            .collect_into_js_array::<ChannelsArray>())
    }

    /// Creates the direct message or joins it if it already exists and returns the `BareJid`.
    /// Sends invites to all participants if the group was created.
    /// Pass a String[] as participants where each string is a valid BareJid.
    #[wasm_bindgen(js_name = "startConversation")]
    pub async fn start_conversation(&self, participants: Array) -> Result<BareJid> {
        let participants = try_user_id_vec_from_string_array(participants)?;

        Ok(self
            .client
            .rooms
            .start_conversation(participants.as_slice())
            .await
            .map_err(WasmError::from)?
            .into_inner()
            .into())
    }

    /// Creates the group or joins it if it already exists and returns the `BareJid`.
    /// Sends invites to all participants if the group was created.
    /// Pass a String[] as participants where each string is a valid BareJid.
    #[wasm_bindgen(js_name = "createGroup")]
    pub async fn create_group(&self, participants: Array) -> Result<BareJid> {
        let participants = try_user_id_vec_from_string_array(participants)?;

        Ok(self
            .client
            .rooms
            .create_room_for_group(participants.as_slice())
            .await
            .map_err(WasmError::from)?
            .into_inner()
            .into())
    }

    /// Creates the public channel or joins it if one with the same name already exists and
    /// returns the `BareJid`.
    #[wasm_bindgen(js_name = "createPublicChannel")]
    pub async fn create_public_channel(&self, channel_name: &str) -> Result<BareJid> {
        Ok(self
            .client
            .rooms
            .create_room_for_public_channel(channel_name)
            .await
            .map_err(WasmError::from)?
            .into_inner()
            .into())
    }

    /// Creates the private channel and returns the `BareJid` of the created room.
    #[wasm_bindgen(js_name = "createPrivateChannel")]
    pub async fn create_private_channel(&self, channel_name: &str) -> Result<BareJid> {
        Ok(self
            .client
            .rooms
            .create_room_for_private_channel(channel_name)
            .await
            .map_err(WasmError::from)?
            .into_inner()
            .into())
    }

    /// Joins the room identified by `room_jid` and returns its `BareJid`.
    #[wasm_bindgen(js_name = "joinRoom")]
    pub async fn join_room(&self, room_jid: &BareJid, password: Option<String>) -> Result<BareJid> {
        Ok(self
            .client
            .rooms
            .join_room(&RoomId::from(room_jid.clone()), password.as_deref())
            .await
            .map_err(|err| WasmError::from(anyhow::Error::from(err)))?
            .into_inner()
            .into())
    }

    /// Destroys the room identified by `room_jid`.
    #[wasm_bindgen(js_name = "destroyRoom")]
    pub async fn destroy_room(&self, room_jid: &BareJid) -> Result<()> {
        self.client
            .rooms
            .destroy_room(&RoomId::from(room_jid.clone()))
            .await
            .map_err(|err| WasmError::from(anyhow::Error::from(err)))?;
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
            Some(UserStatus {
                emoji: icon.clone(),
                status: text.clone(),
            })
        } else {
            None
        };

        self.client
            .account
            .set_user_activity(user_activity)
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    /// Adds a contact to the roster and sends a presence subscription request.
    #[wasm_bindgen(js_name = "addContact")]
    pub async fn add_contact(&self, jid: &BareJid) -> Result<()> {
        Ok(self
            .client
            .contacts
            .add_contact(&jid.into())
            .await
            .map_err(WasmError::from)?)
    }

    #[wasm_bindgen(js_name = "loadContacts")]
    pub async fn load_contacts(&self) -> Result<ContactsArray> {
        Ok(self
            .client
            .contacts
            .load_contacts()
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
            .user_data
            .load_avatar(&jid.into())
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
            .account
            .set_avatar(&image_data, None, None, mime_type)
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
            .user_data
            .load_user_profile(&jid.into())
            .await
            .map_err(WasmError::from)?;

        Ok(profile.map(Into::into))
    }

    /// XEP-0292: vCard4 Over XMPP
    /// https://xmpp.org/extensions/xep-0292.html
    #[wasm_bindgen(js_name = "saveUserProfile")]
    pub async fn save_user_profile(&self, profile: &UserProfile) -> Result<()> {
        self.client
            .account
            .set_profile(&(profile.clone()).into())
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "deleteCachedData")]
    pub async fn delete_cached_data(&self) -> Result<()> {
        self.client
            .cache
            .clear_cache()
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "loadUserMetadata")]
    pub async fn load_user_metadata(&self, jid: &BareJid) -> Result<UserMetadata> {
        let metadata = self
            .client
            .user_data
            .load_user_metadata(&jid.into())
            .await
            .map_err(WasmError::from)?
            .unwrap_or_default();
        Ok(metadata.into())
    }

    /// XMPP: Instant Messaging and Presence
    /// https://xmpp.org/rfcs/rfc6121.html#presence
    #[wasm_bindgen(js_name = "setAvailability")]
    pub async fn set_availability(&self, availability: Availability) -> Result<()> {
        self.client
            .account
            .set_availability(availability.into())
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }
}

impl From<ProseClient> for Client {
    fn from(client: ProseClient) -> Self {
        Client { client }
    }
}
