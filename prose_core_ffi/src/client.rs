use std::fs;
use std::path::{Path, PathBuf};

use tracing::info;

use prose_core_client::{Client as ProseClient, ClientDelegate, FsAvatarCache, SQLiteCache};
use prose_core_lib::ConnectionError;

use crate::{
    BareJid, ClientError, Contact, Emoji, FullJid, Message, MessageId, MessagesPage, UserProfile,
};

pub struct Client {
    jid: FullJid,
    client: ProseClient<SQLiteCache, FsAvatarCache>,
}

impl Client {
    pub fn new(
        jid: FullJid,
        cache_dir: String,
        delegate: Option<Box<dyn ClientDelegate>>,
    ) -> Result<Self, ClientError> {
        let bare_jid: BareJid = jid.clone().into();
        let cache_dir = Path::new(&cache_dir).join(bare_jid.to_string());
        info!("Caching data at {:?}", cache_dir);
        fs::create_dir_all(&cache_dir).map_err(anyhow::Error::new)?;

        Ok(Client {
            jid,
            client: ProseClient::new(
                SQLiteCache::open(&cache_dir)?,
                FsAvatarCache::new(&cache_dir.join("Avatars"))?,
                delegate,
            ),
        })
    }
}

#[uniffi::export]
impl Client {
    pub fn jid(&self) -> FullJid {
        self.jid.clone()
    }

    pub async fn connect(&self, password: String) -> Result<(), ConnectionError> {
        self.client.connect(&self.jid, password).await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), ClientError> {
        self.client.disconnect().await;
        Ok(())
    }

    pub async fn load_contacts(&self) -> Result<Vec<Contact>, ClientError> {
        let items = self.client.load_contacts().await?;
        Ok(items)
    }

    pub async fn load_profile(&self, from: BareJid) -> Result<UserProfile, ClientError> {
        let profile = self.client.load_profile(from).await?;
        Ok(profile)
    }

    pub async fn save_profile(&self, profile: UserProfile) -> Result<(), ClientError> {
        let profile = self.client.save_profile(profile).await?;
        Ok(profile)
    }

    pub async fn load_avatar(&self, from: BareJid) -> Result<Option<PathBuf>, ClientError> {
        let path = self.client.load_avatar(from).await?;
        Ok(path)
    }

    pub async fn save_avatar(&self, image_path: PathBuf) -> Result<(), ClientError> {
        self.client.save_avatar(&image_path).await?;
        Ok(())
    }

    pub async fn load_latest_messages(
        &self,
        from: BareJid,
        since: Option<MessageId>,
        load_from_server: bool,
    ) -> Result<Vec<Message>, ClientError> {
        let messages = self
            .client
            .load_latest_messages(&from, since.as_ref(), load_from_server)
            .await?;
        Ok(messages)
    }

    pub async fn load_messages_before(
        &self,
        from: BareJid,
        before: MessageId,
    ) -> Result<MessagesPage, ClientError> {
        let page = self.client.load_messages_before(&from, &before).await?;
        Ok(page.into())
    }

    pub async fn load_messages_with_ids(
        &self,
        conversation: BareJid,
        ids: Vec<MessageId>,
    ) -> Result<Vec<Message>, ClientError> {
        let messages = self
            .client
            .load_messages_with_ids(&conversation, &ids)
            .await?;
        Ok(messages)
    }

    pub async fn send_message(&self, to: BareJid, body: String) -> Result<(), ClientError> {
        self.client.send_message(to, body).await?;
        Ok(())
    }

    pub async fn update_message(
        &self,
        conversation: BareJid,
        id: MessageId,
        body: String,
    ) -> Result<(), ClientError> {
        self.client.update_message(conversation, id, body).await?;
        Ok(())
    }

    pub async fn toggle_reaction_to_message(
        &self,
        conversation: BareJid,
        id: MessageId,
        emoji: Emoji,
    ) -> Result<(), ClientError> {
        self.client
            .toggle_reaction_to_message(conversation, id, emoji)
            .await?;
        Ok(())
    }

    pub async fn retract_message(
        &self,
        conversation: BareJid,
        id: MessageId,
    ) -> Result<(), ClientError> {
        self.client.retract_message(conversation, id).await?;
        Ok(())
    }

    pub async fn set_user_is_composing(
        &self,
        conversation: BareJid,
        is_composing: bool,
    ) -> Result<(), ClientError> {
        self.client
            .set_user_is_composing(conversation, is_composing)
            .await?;
        Ok(())
    }

    pub async fn load_composing_users(
        &self,
        conversation: BareJid,
    ) -> Result<Vec<BareJid>, ClientError> {
        let users = self.client.load_composing_users(&conversation).await?;
        Ok(users)
    }

    pub async fn save_draft(
        &self,
        conversation: BareJid,
        text: Option<String>,
    ) -> Result<(), ClientError> {
        self.client
            .save_draft(&conversation, text.as_deref())
            .await?;
        Ok(())
    }

    pub async fn load_draft(&self, conversation: BareJid) -> Result<Option<String>, ClientError> {
        let text = self.client.load_draft(&conversation).await?;
        Ok(text)
    }
}
