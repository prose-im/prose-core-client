// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use tracing::info;

use prose_core_client::dtos::{Availability, Emoji, MessageId, UserProfile};
use prose_core_client::infra::encryption::EncryptionKeysRepository;
use prose_core_client::{
    open_store, Client as ProseClient, ClientDelegate as ProseClientDelegate, FsAvatarCache,
    PlatformDriver, Secret, SignalServiceHandle,
};
use prose_xmpp::{connector, ConnectionError};

use crate::types::{ClientEvent, Message, JID};
use crate::{ClientError, Contact};

pub trait ClientDelegate: Send + Sync {
    fn handle_event(&self, event: ClientEvent);
}

pub struct Client {
    jid: JID,
    client: RwLock<Option<ProseClient>>,
    cache_dir: PathBuf,
    delegate: Mutex<Option<Box<dyn ProseClientDelegate>>>,
}

impl Client {
    pub fn new(
        jid: JID,
        cache_dir: String,
        delegate: Option<Box<dyn ClientDelegate>>,
    ) -> Result<Self, ClientError> {
        let cache_dir = Path::new(&cache_dir).join(jid.to_string());

        Ok(Self {
            jid,
            client: Default::default(),
            cache_dir,
            delegate: Mutex::new(
                delegate.map(|d| Box::new(DelegateWrapper(d)) as Box<dyn ProseClientDelegate>),
            ),
        })

        // #[uniffi::export] supports async but doesn't support static methods and
        // the UDL allows static methods but no async methods. Meh.
        // So we need to break this method up so that the async part runs after the
        // constructor in a separate method.

        // let cache_dir = Path::new(&cache_dir).join(jid.to_string());
        // info!("Caching data at {:?}", cache_dir);
        // fs::create_dir_all(&cache_dir).map_err(anyhow::Error::new)?;
        //
        // let delegate = delegate.map(|d| {
        //     Box::new(DelegateWrapper(d))
        //         as Box<dyn ProseClientDelegate<PlatformCache, FsAvatarCache>>
        // });
        //
        // Ok(Client {
        //     jid,
        //     client: ClientBuilder::new()
        //         .set_data_cache(PlatformCache::open(&cache_dir).await?)
        //         .set_avatar_cache(FsAvatarCache::new(&cache_dir.join("Avatars"))?)
        //         .set_delegate(delegate)
        //         .build(),
        // })
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl Client {
    pub fn jid(&self) -> JID {
        self.jid.clone()
    }

    pub async fn connect(&self, password: String) -> Result<(), ConnectionError> {
        self.client()
            .await
            .map_err(|e| ConnectionError::Generic { msg: e.to_string() })?
            .connect(&self.jid.to_bare().unwrap().into(), Secret::new(password))
            .await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), ClientError> {
        self.client().await?.disconnect().await;
        Ok(())
    }

    pub async fn load_contacts(&self) -> Result<Vec<Contact>, ClientError> {
        let items = self.client().await?.contact_list.load_contacts().await?;
        Ok(items.into_iter().map(Into::into).collect())
    }

    pub async fn load_profile(&self, from: JID) -> Result<Option<UserProfile>, ClientError> {
        let profile = self
            .client()
            .await?
            .user_data
            .load_user_profile(&from.to_bare().unwrap().into())
            .await?;
        Ok(profile)
    }

    pub async fn save_profile(&self, profile: UserProfile) -> Result<(), ClientError> {
        let profile = self.client().await?.account.set_profile(&profile).await?;
        Ok(profile)
    }

    pub async fn delete_profile(&self) -> Result<(), ClientError> {
        self.client().await?.account.delete_profile().await?;
        Ok(())
    }

    pub async fn load_avatar(&self, from: JID) -> Result<Option<PathBuf>, ClientError> {
        let path = self
            .client()
            .await?
            .user_data
            .load_avatar(&from.to_bare().unwrap().into())
            .await?;
        Ok(path)
    }

    pub async fn save_avatar(&self, image_path: PathBuf) -> Result<(), ClientError> {
        self.client()
            .await?
            .account
            .set_avatar_from_url(&image_path)
            .await?;
        Ok(())
    }

    pub async fn load_latest_messages(
        &self,
        _from: JID,
        _since: Option<MessageId>,
        _load_from_server: bool,
    ) -> Result<Vec<Message>, ClientError> {
        todo!("Use Room API");
        // let messages = self
        //     .client
        //     .load_latest_messages(&from.into(), since.as_ref(), load_from_server)
        //     .await?;
        // Ok(messages.into_iter().map(Into::into).collect())
    }

    pub async fn load_messages_with_ids(
        &self,
        _conversation: JID,
        _ids: Vec<MessageId>,
    ) -> Result<Vec<Message>, ClientError> {
        todo!("Use Room API");
        // let messages = self
        //     .client
        //     .load_messages_with_ids(&conversation.into(), &ids)
        //     .await?;
        // Ok(messages.into_iter().map(Into::into).collect())
    }

    pub async fn send_message(&self, _to: JID, _body: String) -> Result<(), ClientError> {
        todo!("Use Room API")
        //self.client.send_message(BareJid::from(to), body).await?;
        // Ok(())
    }

    pub async fn update_message(
        &self,
        _conversation: JID,
        _id: MessageId,
        _body: String,
    ) -> Result<(), ClientError> {
        todo!("Use Room API");
        // self.client
        //     .update_message(BareJid::from(conversation), id, body)
        //     .await?;
        // Ok(())
    }

    pub async fn toggle_reaction_to_message(
        &self,
        _conversation: JID,
        _id: MessageId,
        _emoji: Emoji,
    ) -> Result<(), ClientError> {
        todo!("Use Room API");
        // self.client
        //     .toggle_reaction_to_message(BareJid::from(conversation), id, emoji)
        //     .await?;
        // Ok(())
    }

    pub async fn retract_message(
        &self,
        _conversation: JID,
        _id: MessageId,
    ) -> Result<(), ClientError> {
        todo!("Use Room API");
        // self.client
        //     .retract_message(BareJid::from(conversation), id)
        //     .await?;
        // Ok(())
    }

    pub async fn set_user_is_composing(
        &self,
        _conversation: JID,
        _is_composing: bool,
    ) -> Result<(), ClientError> {
        todo!("Use Room API");
        // self.client
        //     .set_user_is_composing(BareJid::from(conversation), is_composing)
        //     .await?;
        // Ok(())
    }

    pub async fn load_composing_users(&self, _conversation: JID) -> Result<Vec<JID>, ClientError> {
        todo!("Use Room API");
        // let users = self
        //     .client
        //     .load_composing_users(&conversation.into())
        //     .await?;
        // Ok(users.into_iter().map(Into::into).collect())
    }

    pub async fn save_draft(
        &self,
        _conversation: JID,
        _text: Option<String>,
    ) -> Result<(), ClientError> {
        todo!("Use Room API");
        // self.client
        //     .save_draft(&conversation.into(), text.as_deref())
        //     .await?;
        // Ok(())
    }

    pub async fn load_draft(&self, _conversation: JID) -> Result<Option<String>, ClientError> {
        todo!("Use Room API");
        // let text = self.client.load_draft(&conversation.into()).await?;
        // Ok(text)
    }

    pub async fn set_availability(&self, availability: Availability) -> Result<(), ClientError> {
        self.client()
            .await?
            .account
            .set_availability(availability)
            .await?;
        Ok(())
    }
}

impl Client {
    async fn client(&self) -> Result<ProseClient, ClientError> {
        if let Some(client) = self.client.read().clone() {
            return Ok(client);
        }

        info!("Caching data at {:?}", self.cache_dir);
        fs::create_dir_all(&self.cache_dir).map_err(anyhow::Error::new)?;

        let store = open_store(PlatformDriver::new(self.cache_dir.join("cache.sqlite")))
            .await
            .map_err(|e| ClientError::Generic { msg: e.to_string() })?;

        let client = ProseClient::builder()
            .set_connector_provider(connector::xmpp_rs::Connector::provider())
            .set_store(store.clone())
            .set_avatar_cache(FsAvatarCache::new(&self.cache_dir.join("Avatars"))?)
            .set_encryption_service(Arc::new(SignalServiceHandle::new(Arc::new(
                EncryptionKeysRepository::new(store),
            ))))
            .set_delegate(self.delegate.lock().take())
            .build();
        self.client.write().replace(client.clone());

        Ok(client)
    }
}

struct DelegateWrapper(Box<dyn ClientDelegate>);

impl ProseClientDelegate for DelegateWrapper {
    fn handle_event(&self, _client: ProseClient, event: prose_core_client::ClientEvent) {
        self.0.handle_event(event.into())
    }
}
