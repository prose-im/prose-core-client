// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::connector::{Connector, ProseConnectionProvider};
use crate::delegate::{Delegate, JSDelegate};
use crate::encryption::{EncryptionService, JsEncryptionService};
use crate::error::{Result, WasmError};
use crate::log::{JSLogger, MakeJSLogWriter};
use crate::types::{
    try_user_id_vec_from_string_array, AccountInfo, Availability, Avatar, BareJid, Channel,
    ChannelsArray, ConnectionError, Contact, ContactsArray, IntoJSArray, PresenceSubRequest,
    PresenceSubRequestArray, PresenceSubRequestId, SidebarItem, SidebarItemsArray, UploadSlot,
    UserBasicInfo, UserBasicInfoArray, UserMetadata, UserProfile, WorkspaceIcon, WorkspaceInfo,
};
use anyhow::anyhow;
use base64::{engine::general_purpose, Engine as _};
use cfg_if::cfg_if;
use js_sys::Array;
use prose_core_client::dtos::{MucId, PlatformImage, SoftwareVersion, UserStatus};
use prose_core_client::infra::encryption::{EncryptionKeysRepository, SessionRepository};
use prose_core_client::{open_store, Client as ProseClient, PlatformDriver, StoreAvatarRepository};
use tracing::{info, Level};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{Blob, BlobPropertyBag};

#[derive(Debug, PartialEq, Clone)]
#[wasm_bindgen(js_name = "ProseClientConfig")]
pub struct ClientConfig {
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
        encryption_service: JsEncryptionService,
        logger: JSLogger,
        config: Option<ClientConfig>,
    ) -> Result<Client> {
        let store = open_store(PlatformDriver::new("ProseDB")).await?;
        let config = config.unwrap_or_default();

        static LOGGING_INITIALIZED: AtomicBool = AtomicBool::new(false);
        if !LOGGING_INITIALIZED.swap(true, Ordering::SeqCst) {
            if config.logging_enabled {
                let fmt_layer = tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .without_time()
                    .with_writer(MakeJSLogWriter::new(logger))
                    .with_level(false)
                    .with_filter(config.logging_min_level.parse().unwrap_or(LevelFilter::OFF));

                tracing_subscriber::registry().with(fmt_layer).init();

                info!("prose-sdk-js Version {VERSION}");
            }
        }

        let software_version = SoftwareVersion {
            name: config.client_name.clone(),
            version: config.client_version.clone(),
            os: config.client_os.clone(),
        };

        cfg_if! {
            if #[cfg(feature = "delay-requests")] {
                tracing::warn!("Using RandomDelayProxyTransformer. Requests will be delayed.");
                let provider: prose_xmpp::client::ConnectorProvider = Box::new({
                    let connection_provider = alloc::rc::Rc::new(connection_provider);
                    move || {
                        let connector = Connector {
                            provider: connection_provider.clone(),
                            config: config.clone(),
                        };
                        Box::new(prose_xmpp::connector::ProxyConnector::new(
                            connector,
                            prose_core_client::RandomDelayProxyTransformer::new(100..2000),
                        ))
                    }
                });
            } else {
                let provider = Connector::provider(connection_provider, config);
            }
        }

        let client = Client {
            client: ProseClient::builder()
                .set_connector_provider(provider)
                .set_store(store.clone())
                .set_avatar_repository(StoreAvatarRepository::new(store.clone()))
                .set_encryption_service(Arc::new(EncryptionService::new(
                    encryption_service,
                    Arc::new(EncryptionKeysRepository::new(store.clone())),
                    Arc::new(SessionRepository::new(store.clone())),
                )))
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
    ) -> std::result::Result<(), ConnectionError> {
        self.client.connect(&jid.into(), password.into()).await?;
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

    /// Returns the `BareJid` of the public room with `name` if one exists.
    #[wasm_bindgen(js_name = "findPublicChannelByName")]
    pub async fn find_public_channel_by_name(&self, name: &str) -> Result<Option<BareJid>> {
        Ok(self
            .client
            .rooms
            .find_public_channel_by_name(name)
            .await
            .map_err(WasmError::from)?
            .map(|room_id| room_id.into_bare().into()))
    }

    #[wasm_bindgen(js_name = "loadAccountInfo")]
    pub async fn load_account_info(&self) -> Result<AccountInfo> {
        Ok(self
            .client
            .account
            .account_info()
            .await
            .map_err(WasmError::from)?
            .into())
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
            .into_bare()
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
            .into_bare()
            .into())
    }

    /// Creates the public channel and returns the `BareJid` of the created room. Fails if another
    /// channel with the same name exists.
    #[wasm_bindgen(js_name = "createPublicChannel")]
    pub async fn create_public_channel(&self, channel_name: &str) -> Result<BareJid> {
        Ok(self
            .client
            .rooms
            .create_room_for_public_channel(channel_name)
            .await
            .map_err(WasmError::from)?
            .into_bare()
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
            .into_bare()
            .into())
    }

    /// Joins the room identified by `room_jid` and returns its `BareJid`.
    #[wasm_bindgen(js_name = "joinRoom")]
    pub async fn join_room(&self, room_jid: &BareJid, password: Option<String>) -> Result<BareJid> {
        Ok(self
            .client
            .rooms
            .join_room(&MucId::from(room_jid.clone()), password.as_deref())
            .await
            .map_err(|err| WasmError::from(anyhow::Error::from(err)))?
            .into_bare()
            .into())
    }

    /// Destroys the room identified by `room_jid`.
    #[wasm_bindgen(js_name = "destroyRoom")]
    pub async fn destroy_room(&self, room_jid: &BareJid) -> Result<()> {
        self.client
            .rooms
            .destroy_room(&MucId::from(room_jid.clone()))
            .await
            .map_err(|err| WasmError::from(anyhow::Error::from(err)))?;
        Ok(())
    }

    /// XEP-0077: In-Band Registration
    /// https://xmpp.org/extensions/xep-0077.html#usecases-changepw
    #[wasm_bindgen(js_name = "changePassword")]
    pub async fn change_password(&self, new_password: &str) -> Result<()> {
        Ok(self
            .client
            .account
            .change_password(new_password)
            .await
            .map_err(WasmError::from)?)
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
            .contact_list
            .add_contact(&jid.into())
            .await
            .map_err(WasmError::from)?)
    }

    /// Removes a contact from the roster
    #[wasm_bindgen(js_name = "removeContact")]
    pub async fn remove_contact(&self, jid: &BareJid) -> Result<()> {
        Ok(self
            .client
            .contact_list
            .remove_contact(&jid.into())
            .await
            .map_err(WasmError::from)?)
    }

    #[wasm_bindgen(js_name = "loadContacts")]
    pub async fn load_contacts(&self) -> Result<ContactsArray> {
        Ok(self
            .client
            .contact_list
            .load_contacts()
            .await
            .map_err(WasmError::from)?
            .into_iter()
            .map(|c| JsValue::from(Contact::from(c)))
            .collect_into_js_array::<ContactsArray>())
    }

    /// Requests a presence subscription from `jid`. Note that happens automatically when you
    /// call `add_contact`. This method can be useful though when our user needs to re-request
    /// the presence subscription in case the contact hasn't reacted in a while.
    #[wasm_bindgen(js_name = "requestPresenceSubscription")]
    pub async fn request_presence_sub(&self, jid: &BareJid) -> Result<()> {
        self.client
            .contact_list
            .request_presence_sub(&jid.into())
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    /// Loads pending presence subscription requests.
    #[wasm_bindgen(js_name = "loadPresenceSubscriptionRequests")]
    pub async fn load_presence_sub_requests(&self) -> Result<PresenceSubRequestArray> {
        Ok(self
            .client
            .contact_list
            .load_presence_sub_requests()
            .await
            .map_err(WasmError::from)?
            .into_iter()
            .map(PresenceSubRequest::from)
            .collect_into_js_array::<PresenceSubRequestArray>())
    }

    /// Approves the presence subscription request identified by `id`.
    #[wasm_bindgen(js_name = "approvePresenceSubscriptionRequest")]
    pub async fn approve_presence_sub_request(&self, id: &PresenceSubRequestId) -> Result<()> {
        self.client
            .contact_list
            .approve_presence_sub_request(id.as_ref())
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    /// Denies the presence subscription request identified by `id`.
    #[wasm_bindgen(js_name = "denyPresenceSubscriptionRequest")]
    pub async fn deny_presence_sub_request(&self, id: &PresenceSubRequestId) -> Result<()> {
        self.client
            .contact_list
            .deny_presence_sub_request(id.as_ref())
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    /// This method is deprecated. Use `loadWorkspaceIconBlob` instead.
    #[wasm_bindgen(js_name = "loadWorkspaceIcon")]
    pub async fn load_workspace_icon(&self, icon: &WorkspaceIcon) -> Result<Option<String>> {
        let icon = self
            .client
            .workspace
            .load_workspace_icon(&icon.clone().into())
            .await
            .map_err(WasmError::from)?;
        Ok(icon.map(|img| img.base64()))
    }

    #[wasm_bindgen(js_name = "loadWorkspaceIconBlob")]
    pub async fn load_workspace_icon_blob(&self, icon: &WorkspaceIcon) -> Result<Option<Blob>> {
        let icon = self
            .client
            .workspace
            .load_workspace_icon(&icon.clone().into())
            .await
            .map_err(WasmError::from)?;
        Ok(icon.map(|img| img.to_blob()).transpose()?)
    }

    #[wasm_bindgen(js_name = "loadWorkspaceInfo")]
    pub async fn load_workspace_info(&self) -> Result<WorkspaceInfo> {
        let info = self
            .client
            .workspace
            .load_workspace_info()
            .await
            .map_err(WasmError::from)?;
        Ok(info.into())
    }

    /// This method is deprecated. Use `loadAvatarBlob` instead.
    #[wasm_bindgen(js_name = "loadAvatarDataURL")]
    pub async fn load_avatar_data_url(&self, avatar: &Avatar) -> Result<Option<String>> {
        let avatar = self
            .client
            .user_data
            .load_avatar(&avatar.clone().into())
            .await
            .map_err(WasmError::from)?;
        Ok(avatar.map(|avatar| avatar.base64()))
    }

    /// XEP-0084: User Avatar
    /// https://xmpp.org/extensions/xep-0084.html
    #[wasm_bindgen(js_name = "loadAvatarBlob")]
    pub async fn load_avatar_blob(&self, avatar: &Avatar) -> Result<Option<Blob>> {
        let Some(img) = self
            .client
            .user_data
            .load_avatar(&avatar.clone().into())
            .await
            .map_err(WasmError::from)?
        else {
            return Ok(None);
        };

        img.to_blob().map(Some)
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
            .set_profile(profile.clone().into())
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

    /// Returns the list of blocked users.
    #[wasm_bindgen(js_name = "loadBlockList")]
    pub async fn load_block_list(&self) -> Result<UserBasicInfoArray> {
        let block_list = self
            .client
            .block_list
            .load_block_list()
            .await
            .map_err(WasmError::from)?
            .into_iter()
            .map(UserBasicInfo::from)
            .collect_into_js_array::<UserBasicInfoArray>();
        Ok(block_list)
    }

    /// Blocks the user identified by `jid`.
    #[wasm_bindgen(js_name = "blockUser")]
    pub async fn block_user(&self, jid: &BareJid) -> Result<()> {
        self.client
            .block_list
            .block_user(&jid.into())
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    /// Unblocks the user identified by `jid`.
    #[wasm_bindgen(js_name = "unblockUser")]
    pub async fn unblock_user(&self, jid: &BareJid) -> Result<()> {
        self.client
            .block_list
            .unblock_user(&jid.into())
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    /// Removes all users from the block list.
    #[wasm_bindgen(js_name = "clearBlockList")]
    pub async fn clear_block_list(&self) -> Result<()> {
        self.client
            .block_list
            .clear_block_list()
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    /// Request a slot for uploading a file to attach it to a message.
    #[wasm_bindgen(js_name = "requestUploadSlot")]
    pub async fn request_upload_slot(
        &self,
        file_name: &str,
        file_size: u64,
        media_type: Option<String>,
    ) -> Result<UploadSlot> {
        let media_type = media_type
            .map(|mt| mt.parse())
            .transpose()
            .map_err(|err| WasmError::from(anyhow!("{err}")))?;

        let slot = self
            .client
            .uploads
            .request_upload_slot(file_name, file_size, media_type)
            .await
            .map_err(WasmError::from)?;
        Ok(slot.into())
    }

    /// Converts a Markdown string to HTML for preview purposes.
    #[wasm_bindgen(js_name = "previewMarkdown")]
    pub fn preview_markdown(&self, markdown: &str) -> String {
        self.client.preview.preview_markdown(markdown)
    }
}

impl From<ProseClient> for Client {
    fn from(client: ProseClient) -> Self {
        Client { client }
    }
}

trait PlatformImageExt {
    fn to_blob(&self) -> Result<Blob>;
}

impl PlatformImageExt for PlatformImage {
    fn to_blob(&self) -> Result<Blob> {
        let props = BlobPropertyBag::new();
        props.set_type(&self.mime_type);

        // SAFETY: The slice will live for the duration of this function call,
        // and `new Blob()` will not modify the bytes or keep a reference to them past the end of the call.
        let uint8_array = unsafe { js_sys::Uint8Array::view(&self.data[..]) };
        let parts = Array::of1(&uint8_array);

        Blob::new_with_u8_array_sequence_and_options(&parts, &props)
            .map_err(|_| JsError::new("Could not initialize blob."))
    }
}
