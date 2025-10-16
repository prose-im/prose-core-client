// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::types::{
    AccountInfo, Availability, Avatar, ClientResult, ConnectionError, PresenceSubRequest,
    PublicRoomInfo, SidebarItem, UploadSlot, UserBasicInfo, UserMetadata, UserProfile, UserStatus,
    WorkspaceIcon, WorkspaceInfo,
};
use crate::{ClientEvent, Contact, Mime, MucId, PathBuf, PresenceSubRequestId, RoomId, UserId};
use prose_core_client::dtos::{SoftwareVersion, UserId as CoreUserId};
use prose_core_client::infra::encryption::{EncryptionKeysRepository, SessionRepository};
use prose_core_client::infra::general::OsRngProvider;
use prose_core_client::{
    open_store, Client as CoreClient, ClientDelegate as CoreClientDelegate,
    ClientEvent as CoreClientEvent, FsAvatarRepository, PlatformDriver, SignalServiceHandle,
};
use prose_xmpp::connector;
use tracing::info;
use tracing::metadata::LevelFilter;
use tracing::Level;
use tracing_oslog::OsLogger;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, Registry};

#[uniffi::export(with_foreign)]
pub trait ClientDelegate: Send + Sync {
    fn handle_event(&self, event: ClientEvent);
}

#[derive(uniffi::Record)]
pub struct ClientConfig {
    #[uniffi(default = false)]
    pub log_received_stanzas: bool,
    #[uniffi(default = false)]
    pub log_sent_stanzas: bool,
    #[uniffi(default = false)]
    pub logging_enabled: bool,
    #[uniffi(default = "trace")]
    pub logging_min_level: String,
    pub client_name: String,
    pub client_version: String,
    pub client_os: Option<String>,
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

#[derive(uniffi::Object)]
pub struct Client {
    client: CoreClient,
}

#[uniffi::export]
impl Client {
    #[uniffi::constructor(async_runtime = "tokio")]
    pub async fn new(
        cache_dir: PathBuf,
        delegate: Option<Arc<dyn ClientDelegate>>,
        config: Option<ClientConfig>,
    ) -> ClientResult<Self> {
        let oslog_layer = OsLogger::new("org.prose", "default")
            .with_filter(LevelFilter::from_level(Level::TRACE));

        Registry::default().with(oslog_layer).init();

        let cache_path = cache_dir.into_inner();
        let cache_dir = Path::new(&cache_path);
        info!("Caching data at {:?}", cache_dir);
        fs::create_dir_all(&cache_dir).map_err(anyhow::Error::new)?;

        let delegate =
            delegate.map(|d| Box::new(DelegateWrapper(d)) as Box<dyn CoreClientDelegate>);

        let store = open_store(PlatformDriver::new(cache_dir.join("ProseDB.sqlite"))).await?;
        let config = config.unwrap_or_default();

        let software_version = SoftwareVersion {
            name: config.client_name.clone(),
            version: config.client_version.clone(),
            os: config.client_os.clone(),
        };

        let client = CoreClient::builder()
            .set_connector_provider(connector::xmpp_rs::Connector::provider())
            .set_store(store.clone())
            .set_avatar_repository(FsAvatarRepository::new(&cache_dir.join("Avatars"))?)
            .set_encryption_service(Arc::new(SignalServiceHandle::new(
                Arc::new(EncryptionKeysRepository::new(store.clone())),
                Arc::new(SessionRepository::new(store)),
                Arc::new(OsRngProvider),
            )))
            .set_delegate(delegate)
            .set_software_version(software_version)
            .build();

        Ok(Client { client })
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl Client {
    pub async fn connect(&self, user_id: UserId, password: String) -> Result<(), ConnectionError> {
        self.client
            .connect(&(user_id.into()), password.into())
            .await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> ClientResult<()> {
        self.client.disconnect().await;
        Ok(())
    }

    pub async fn start_observing_rooms(&self) -> ClientResult<()> {
        self.client.rooms.start_observing_rooms().await?;
        Ok(())
    }

    pub async fn sidebar_items(&self) -> Vec<SidebarItem> {
        self.client
            .sidebar
            .sidebar_items()
            .await
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub async fn load_public_channels(&self) -> ClientResult<Vec<PublicRoomInfo>> {
        Ok(self
            .client
            .rooms
            .load_public_rooms()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub async fn find_public_channel_by_name(&self, name: &str) -> ClientResult<Option<RoomId>> {
        Ok(self
            .client
            .rooms
            .find_public_channel_by_name(name)
            .await?
            .map(Into::into))
    }

    pub async fn load_account_info(&self) -> ClientResult<AccountInfo> {
        Ok(self.client.account.account_info().await?.into())
    }

    /// Creates the direct message or joins it if it already exists and returns the `BareJid`.
    /// Sends invites to all participants if the group was created.
    pub async fn start_conversation(&self, participants: Vec<UserId>) -> ClientResult<RoomId> {
        let participants = participants
            .into_iter()
            .map(Into::into)
            .collect::<Vec<CoreUserId>>();
        Ok(self
            .client
            .rooms
            .start_conversation(participants.as_slice())
            .await?
            .into())
    }

    /// Creates the group or joins it if it already exists and returns the `BareJid`.
    /// Sends invites to all participants if the group was created.
    pub async fn create_group(&self, participants: Vec<UserId>) -> ClientResult<RoomId> {
        let participants = participants
            .into_iter()
            .map(Into::into)
            .collect::<Vec<CoreUserId>>();
        Ok(self
            .client
            .rooms
            .create_room_for_group(participants.as_slice())
            .await?
            .into())
    }

    /// Creates the public channel and returns the `BareJid` of the created room. Fails if another
    /// channel with the same name exists.
    pub async fn create_public_channel(&self, channel_name: &str) -> ClientResult<RoomId> {
        Ok(self
            .client
            .rooms
            .create_room_for_public_channel(channel_name)
            .await?
            .into())
    }

    /// Creates the private channel and returns the `BareJid` of the created room.
    pub async fn create_private_channel(&self, channel_name: &str) -> ClientResult<RoomId> {
        Ok(self
            .client
            .rooms
            .create_room_for_private_channel(channel_name)
            .await?
            .into())
    }

    /// Joins the room identified by `room_jid` and returns its `BareJid`.
    pub async fn join_room(
        &self,
        room_id: MucId,
        password: Option<String>,
    ) -> ClientResult<RoomId> {
        Ok(self
            .client
            .rooms
            .join_room(&(room_id.into()), password.as_deref())
            .await?
            .into())
    }

    /// Destroys the room identified by `room_jid`.
    pub async fn destroy_room(&self, room_id: MucId) -> ClientResult<()> {
        self.client.rooms.destroy_room(&(room_id.into())).await?;
        Ok(())
    }

    /// XEP-0077: In-Band Registration
    /// https://xmpp.org/extensions/xep-0077.html#usecases-changepw
    pub async fn change_password(&self, new_password: &str) -> ClientResult<()> {
        self.client.account.change_password(new_password).await?;
        Ok(())
    }

    /// XEP-0108: User Activity
    /// https://xmpp.org/extensions/xep-0108.html
    pub async fn set_user_activity(&self, status: Option<UserStatus>) -> ClientResult<()> {
        self.client
            .account
            .set_user_activity(status.map(Into::into))
            .await?;
        Ok(())
    }

    /// Adds a contact to the roster and sends a presence subscription request.
    pub async fn add_contact(&self, user_id: UserId) -> ClientResult<()> {
        self.client
            .contact_list
            .add_contact(&(user_id.into()))
            .await?;
        Ok(())
    }

    /// Removes a contact from the roster
    pub async fn remove_contact(&self, user_id: UserId) -> ClientResult<()> {
        self.client
            .contact_list
            .remove_contact(&(user_id.into()))
            .await?;
        Ok(())
    }

    pub async fn load_contacts(&self) -> ClientResult<Vec<Contact>> {
        Ok(self
            .client
            .contact_list
            .load_contacts()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    /// Requests a presence subscription from `jid`. Note that happens automatically when you
    /// call `add_contact`. This method can be useful though when our user needs to re-request
    /// the presence subscription in case the contact hasn't reacted in a while.
    pub async fn request_presence_sub(&self, user_id: UserId) -> ClientResult<()> {
        self.client
            .contact_list
            .request_presence_sub(&(user_id.into()))
            .await?;
        Ok(())
    }

    /// Loads pending presence subscription requests.
    pub async fn load_presence_sub_requests(&self) -> ClientResult<Vec<PresenceSubRequest>> {
        Ok(self
            .client
            .contact_list
            .load_presence_sub_requests()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    /// Approves the presence subscription request identified by `id`.
    pub async fn approve_presence_sub_request(&self, id: PresenceSubRequestId) -> ClientResult<()> {
        self.client
            .contact_list
            .approve_presence_sub_request(&(id.into()))
            .await?;
        Ok(())
    }

    /// Denies the presence subscription request identified by `id`.
    pub async fn deny_presence_sub_request(&self, id: PresenceSubRequestId) -> ClientResult<()> {
        self.client
            .contact_list
            .deny_presence_sub_request(&(id.into()))
            .await?;
        Ok(())
    }

    pub async fn load_workspace_icon(&self, icon: WorkspaceIcon) -> ClientResult<Option<PathBuf>> {
        Ok(self
            .client
            .workspace
            .load_workspace_icon(&(icon.into()))
            .await?
            .map(Into::into))
    }

    pub async fn load_workspace_info(&self) -> ClientResult<WorkspaceInfo> {
        Ok(self.client.workspace.load_workspace_info().await?.into())
    }

    /// XEP-0084: User Avatar
    /// https://xmpp.org/extensions/xep-0084.html
    pub async fn load_avatar(&self, avatar: Arc<Avatar>) -> ClientResult<Option<PathBuf>> {
        Ok(self
            .client
            .user_data
            .load_avatar((*avatar).as_ref())
            .await?
            .map(Into::into))
    }

    /// XEP-0084: User Avatar
    /// https://xmpp.org/extensions/xep-0084.html
    pub async fn save_avatar(&self, image_path: PathBuf) -> ClientResult<()> {
        self.client
            .account
            .set_avatar_from_url(image_path.into_inner())
            .await?;
        Ok(())
    }

    /// XEP-0292: vCard4 Over XMPP
    /// https://xmpp.org/extensions/xep-0292.html
    pub async fn load_profile(&self, from: UserId) -> ClientResult<Option<UserProfile>> {
        Ok(self
            .client
            .user_data
            .load_user_profile(&(from.into()))
            .await?
            .map(Into::into))
    }

    /// XEP-0292: vCard4 Over XMPP
    /// https://xmpp.org/extensions/xep-0292.html
    pub async fn save_profile(&self, profile: UserProfile) -> ClientResult<()> {
        self.client.account.set_profile(profile.into()).await?;
        Ok(())
    }

    pub async fn delete_cached_data(&self) -> ClientResult<()> {
        self.client.cache.clear_cache().await?;
        Ok(())
    }

    pub async fn load_user_metadata(&self, user_id: UserId) -> ClientResult<UserMetadata> {
        Ok(self
            .client
            .user_data
            .load_user_metadata(&(user_id.into()))
            .await?
            .unwrap_or_default()
            .into())
    }

    /// XMPP: Instant Messaging and Presence
    /// https://xmpp.org/rfcs/rfc6121.html#presence
    pub async fn set_availability(&self, availability: Availability) -> ClientResult<()> {
        self.client
            .account
            .set_availability(availability.into())
            .await?;
        Ok(())
    }

    /// Returns the list of blocked users.
    pub async fn load_block_list(&self) -> ClientResult<Vec<UserBasicInfo>> {
        Ok(self
            .client
            .block_list
            .load_block_list()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    /// Blocks the user identified by `user_id`.
    pub async fn block_user(&self, user_id: UserId) -> ClientResult<()> {
        self.client.block_list.block_user(&user_id.into()).await?;
        Ok(())
    }

    /// Unblocks the user identified by `user_id`.
    pub async fn unblock_user(&self, user_id: UserId) -> ClientResult<()> {
        self.client.block_list.unblock_user(&user_id.into()).await?;
        Ok(())
    }

    /// Removes all users from the block list.
    pub async fn clear_block_list(&self) -> ClientResult<()> {
        self.client.block_list.clear_block_list().await?;
        Ok(())
    }

    /// Request a slot for uploading a file to attach it to a message.
    pub async fn request_upload_slot(
        &self,
        file_name: &str,
        file_size: u64,
        media_type: Option<Mime>,
    ) -> ClientResult<UploadSlot> {
        Ok(self
            .client
            .uploads
            .request_upload_slot(file_name, file_size, media_type.map(Into::into))
            .await?
            .into())
    }

    pub fn preview_markdown(&self, markdown: &str) -> String {
        self.client.preview.preview_markdown(markdown)
    }
}

struct DelegateWrapper(Arc<dyn ClientDelegate>);

impl CoreClientDelegate for DelegateWrapper {
    fn handle_event(&self, _client: CoreClient, event: CoreClientEvent) {
        self.0.handle_event(event.into())
    }
}
