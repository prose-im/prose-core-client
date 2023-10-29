// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::path::PathBuf;

use jid::BareJid;

use prose_core_client::dtos::{Availability, MessageId, UserProfile};
use prose_core_client::{Client as ProseClient, ClientDelegate as ProseClientDelegate};
use prose_xmpp::stanza::message::Emoji;
use prose_xmpp::ConnectionError;

use crate::types::{ClientEvent, Message, JID};
use crate::{ClientError, Contact};

pub trait ClientDelegate: Send + Sync {
    fn handle_event(&self, event: ClientEvent);
}

pub struct Client {
    jid: JID,
    client: ProseClient,
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

    pub async fn connect(&self, password: String) -> Result<(), ConnectionError> {
        self.client
            .connect(&self.jid.to_bare().unwrap(), password)
            .await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), ClientError> {
        self.client.disconnect().await;
        Ok(())
    }

    pub async fn load_contacts(&self) -> Result<Vec<Contact>, ClientError> {
        let items = self.client.contacts.load_contacts().await?;
        Ok(items.into_iter().map(Into::into).collect())
    }

    pub async fn load_profile(&self, from: JID) -> Result<Option<UserProfile>, ClientError> {
        let profile = self
            .client
            .user_data
            .load_user_profile(&from.to_bare().unwrap())
            .await?;
        Ok(profile)
    }

    pub async fn save_profile(&self, profile: UserProfile) -> Result<(), ClientError> {
        let profile = self.client.account.set_profile(&profile).await?;
        Ok(profile)
    }

    pub async fn delete_profile(&self) -> Result<(), ClientError> {
        self.client.account.delete_profile().await?;
        Ok(())
    }

    pub async fn load_avatar(&self, from: JID) -> Result<Option<PathBuf>, ClientError> {
        let path = self
            .client
            .user_data
            .load_avatar(&from.to_bare().unwrap())
            .await?;
        Ok(path)
    }

    pub async fn save_avatar(&self, image_path: PathBuf) -> Result<(), ClientError> {
        self.client.account.set_avatar_from_url(&image_path).await?;
        Ok(())
    }

    pub async fn load_latest_messages(
        &self,
        from: JID,
        since: Option<MessageId>,
        load_from_server: bool,
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
        conversation: JID,
        ids: Vec<MessageId>,
    ) -> Result<Vec<Message>, ClientError> {
        todo!("Use Room API");
        // let messages = self
        //     .client
        //     .load_messages_with_ids(&conversation.into(), &ids)
        //     .await?;
        // Ok(messages.into_iter().map(Into::into).collect())
    }

    pub async fn send_message(&self, to: JID, body: String) -> Result<(), ClientError> {
        todo!("Use Room API")
        //self.client.send_message(BareJid::from(to), body).await?;
        // Ok(())
    }

    pub async fn update_message(
        &self,
        conversation: JID,
        id: MessageId,
        body: String,
    ) -> Result<(), ClientError> {
        todo!("Use Room API");
        // self.client
        //     .update_message(BareJid::from(conversation), id, body)
        //     .await?;
        Ok(())
    }

    pub async fn toggle_reaction_to_message(
        &self,
        conversation: JID,
        id: MessageId,
        emoji: Emoji,
    ) -> Result<(), ClientError> {
        todo!("Use Room API");
        // self.client
        //     .toggle_reaction_to_message(BareJid::from(conversation), id, emoji)
        //     .await?;
        // Ok(())
    }

    pub async fn retract_message(
        &self,
        conversation: JID,
        id: MessageId,
    ) -> Result<(), ClientError> {
        todo!("Use Room API");
        // self.client
        //     .retract_message(BareJid::from(conversation), id)
        //     .await?;
        // Ok(())
    }

    pub async fn set_user_is_composing(
        &self,
        conversation: JID,
        is_composing: bool,
    ) -> Result<(), ClientError> {
        todo!("Use Room API");
        // self.client
        //     .set_user_is_composing(BareJid::from(conversation), is_composing)
        //     .await?;
        // Ok(())
    }

    pub async fn load_composing_users(&self, conversation: JID) -> Result<Vec<JID>, ClientError> {
        todo!("Use Room API");
        // let users = self
        //     .client
        //     .load_composing_users(&conversation.into())
        //     .await?;
        // Ok(users.into_iter().map(Into::into).collect())
    }

    pub async fn save_draft(
        &self,
        conversation: JID,
        text: Option<String>,
    ) -> Result<(), ClientError> {
        todo!("Use Room API");
        // self.client
        //     .save_draft(&conversation.into(), text.as_deref())
        //     .await?;
        // Ok(())
    }

    pub async fn load_draft(&self, conversation: JID) -> Result<Option<String>, ClientError> {
        todo!("Use Room API");
        // let text = self.client.load_draft(&conversation.into()).await?;
        // Ok(text)
    }

    pub async fn set_availability(&self, availability: Availability) -> Result<(), ClientError> {
        self.client.account.set_availability(availability).await?;
        Ok(())
    }
}

struct DelegateWrapper(Box<dyn ClientDelegate>);

impl ProseClientDelegate for DelegateWrapper {
    fn handle_event(&self, _client: ProseClient, event: prose_core_client::ClientEvent) {
        self.0.handle_event(event.into())
    }
}
