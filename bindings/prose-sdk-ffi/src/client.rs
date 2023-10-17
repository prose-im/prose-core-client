// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;
use prose_core_client::data_cache::indexed_db::PlatformCache;
use prose_core_client::types::AccountSettings;
use prose_core_client::{
    CachePolicy, Client as ProseClient, ClientBuilder, ClientDelegate as ProseClientDelegate,
    FsAvatarCache,
};
use prose_xmpp::ConnectionError;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

use crate::types::{ClientEvent, Message, MessagesPage, JID};
use crate::{Availability, ClientError, Contact, Emoji, MessageId, UserProfile};

pub trait ClientDelegate: Send + Sync {
    fn handle_event(&self, event: ClientEvent);
}

pub struct Client {
    jid: JID,
    client: ProseClient<PlatformCache, FsAvatarCache>,
}

impl Client {
    pub fn new(
        jid: JID,
        cache_dir: String,
        delegate: Option<Box<dyn ClientDelegate>>,
    ) -> Result<Self, ClientError> {
        todo!("FIXME")

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

#[uniffi::export]
impl Client {
    pub fn jid(&self) -> JID {
        self.jid.clone()
    }

    pub async fn connect(
        &self,
        password: String,
        availability: Availability,
    ) -> Result<(), ConnectionError> {
        // TODO: Generate and store resource.
        let full_jid = self.jid.to_full_jid_with_resource("macOS").unwrap();

        self.client
            .connect(&full_jid, password, availability)
            .await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), ClientError> {
        self.client.disconnect().await;
        Ok(())
    }

    pub async fn load_contacts(
        &self,
        cache_policy: CachePolicy,
    ) -> Result<Vec<Contact>, ClientError> {
        let items = self.client.load_contacts(cache_policy).await?;
        Ok(items.into_iter().map(Into::into).collect())
    }

    pub async fn load_profile(
        &self,
        from: JID,
        cache_policy: CachePolicy,
    ) -> Result<Option<UserProfile>, ClientError> {
        let profile = self.client.load_user_profile(from, cache_policy).await?;
        Ok(profile)
    }

    pub async fn save_profile(&self, profile: UserProfile) -> Result<(), ClientError> {
        let profile = self.client.save_profile(profile).await?;
        Ok(profile)
    }

    pub async fn delete_profile(&self) -> Result<(), ClientError> {
        self.client.delete_profile().await?;
        Ok(())
    }

    pub async fn load_avatar(
        &self,
        from: JID,
        cache_policy: CachePolicy,
    ) -> Result<Option<PathBuf>, ClientError> {
        let path = self.client.load_avatar(from, cache_policy).await?;
        Ok(path)
    }

    pub async fn save_avatar(&self, image_path: PathBuf) -> Result<(), ClientError> {
        self.client.save_avatar_from_url(&image_path).await?;
        Ok(())
    }

    pub async fn load_latest_messages(
        &self,
        from: JID,
        since: Option<MessageId>,
        load_from_server: bool,
    ) -> Result<Vec<Message>, ClientError> {
        let messages = self
            .client
            .load_latest_messages(&from.into(), since.as_ref(), load_from_server)
            .await?;
        Ok(messages.into_iter().map(Into::into).collect())
    }

    pub async fn load_messages_before(
        &self,
        from: JID,
        before: MessageId,
    ) -> Result<MessagesPage, ClientError> {
        let page = self
            .client
            .load_messages_before(&from.into(), &before)
            .await?;
        Ok(page.into())
    }

    pub async fn load_messages_with_ids(
        &self,
        conversation: JID,
        ids: Vec<MessageId>,
    ) -> Result<Vec<Message>, ClientError> {
        let messages = self
            .client
            .load_messages_with_ids(&conversation.into(), &ids)
            .await?;
        Ok(messages.into_iter().map(Into::into).collect())
    }

    pub async fn send_message(&self, to: JID, body: String) -> Result<(), ClientError> {
        self.client.send_message(BareJid::from(to), body).await?;
        Ok(())
    }

    pub async fn update_message(
        &self,
        conversation: JID,
        id: MessageId,
        body: String,
    ) -> Result<(), ClientError> {
        self.client
            .update_message(BareJid::from(conversation), id, body)
            .await?;
        Ok(())
    }

    pub async fn toggle_reaction_to_message(
        &self,
        conversation: JID,
        id: MessageId,
        emoji: Emoji,
    ) -> Result<(), ClientError> {
        self.client
            .toggle_reaction_to_message(BareJid::from(conversation), id, emoji)
            .await?;
        Ok(())
    }

    pub async fn retract_message(
        &self,
        conversation: JID,
        id: MessageId,
    ) -> Result<(), ClientError> {
        self.client
            .retract_message(BareJid::from(conversation), id)
            .await?;
        Ok(())
    }

    pub async fn set_user_is_composing(
        &self,
        conversation: JID,
        is_composing: bool,
    ) -> Result<(), ClientError> {
        self.client
            .set_user_is_composing(BareJid::from(conversation), is_composing)
            .await?;
        Ok(())
    }

    pub async fn load_composing_users(&self, conversation: JID) -> Result<Vec<JID>, ClientError> {
        let users = self
            .client
            .load_composing_users(&conversation.into())
            .await?;
        Ok(users.into_iter().map(Into::into).collect())
    }

    pub async fn save_draft(
        &self,
        conversation: JID,
        text: Option<String>,
    ) -> Result<(), ClientError> {
        self.client
            .save_draft(&conversation.into(), text.as_deref())
            .await?;
        Ok(())
    }

    pub async fn load_draft(&self, conversation: JID) -> Result<Option<String>, ClientError> {
        let text = self.client.load_draft(&conversation.into()).await?;
        Ok(text)
    }

    pub async fn set_availability(&self, availability: Availability) -> Result<(), ClientError> {
        self.client.set_availability(availability).await?;
        Ok(())
    }

    pub async fn load_account_settings(&self) -> Result<AccountSettings, ClientError> {
        let settings = self.client.load_account_settings().await?;
        Ok(settings)
    }

    pub async fn save_account_settings(
        &self,
        settings: AccountSettings,
    ) -> Result<(), ClientError> {
        self.client.save_account_settings(&settings).await?;
        Ok(())
    }
}

struct DelegateWrapper(Box<dyn ClientDelegate>);

impl ProseClientDelegate<PlatformCache, FsAvatarCache> for DelegateWrapper {
    fn handle_event(
        &self,
        _client: ProseClient<PlatformCache, FsAvatarCache>,
        event: prose_core_client::ClientEvent<PlatformCache, FsAvatarCache>,
    ) {
        self.0.handle_event(event.into())
    }
}
