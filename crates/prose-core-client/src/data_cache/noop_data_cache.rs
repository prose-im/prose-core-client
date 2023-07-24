use async_trait::async_trait;
use chrono::{DateTime, Utc};
use jid::BareJid;
use thiserror::Error;

use prose_domain::UserProfile;
use prose_xmpp::stanza::avatar::ImageId;
use prose_xmpp::stanza::message::{ChatState, Id};
use prose_xmpp::SendUnlessWasm;

use crate::data_cache::{ContactsCache, DataCache, MessageCache};
use crate::types::roster::Item;
use crate::types::{
    AccountSettings, AvatarMetadata, Contact, MessageLike, Page, Presence, UserActivity,
};

#[derive(Error, Debug)]
#[error(transparent)]
pub struct NoopDataCacheError(#[from] anyhow::Error);

type Result<T> = std::result::Result<T, NoopDataCacheError>;

pub struct NoopDataCache {}

impl Default for NoopDataCache {
    fn default() -> Self {
        NoopDataCache {}
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ContactsCache for NoopDataCache {
    type Error = NoopDataCacheError;

    async fn set_roster_update_time(
        &self,
        _timestamp: &DateTime<Utc>,
    ) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    async fn roster_update_time(&self) -> std::result::Result<Option<DateTime<Utc>>, Self::Error> {
        Ok(None)
    }

    async fn insert_roster_items(&self, _items: &[Item]) -> Result<()> {
        Ok(())
    }

    async fn insert_user_profile(&self, _jid: &BareJid, _profile: &UserProfile) -> Result<()> {
        Ok(())
    }

    async fn load_user_profile(&self, _jid: &BareJid) -> Result<Option<UserProfile>> {
        Ok(None)
    }

    async fn delete_user_profile(&self, _jid: &BareJid) -> Result<()> {
        Ok(())
    }

    async fn insert_avatar_metadata(
        &self,
        _jid: &BareJid,
        _metadata: &AvatarMetadata,
    ) -> Result<()> {
        Ok(())
    }

    async fn load_avatar_metadata(&self, _jid: &BareJid) -> Result<Option<AvatarMetadata>> {
        Ok(None)
    }

    async fn insert_presence(&self, _jid: &BareJid, _presence: &Presence) -> Result<()> {
        Ok(())
    }
    async fn insert_user_activity(
        &self,
        _jid: &BareJid,
        _user_activity: &Option<UserActivity>,
    ) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    async fn insert_chat_state(&self, _jid: &BareJid, _chat_state: &ChatState) -> Result<()> {
        Ok(())
    }

    async fn load_chat_state(&self, _jid: &BareJid) -> Result<Option<ChatState>> {
        Ok(None)
    }

    async fn load_contacts(&self) -> Result<Vec<(Contact, Option<ImageId>)>> {
        Ok(vec![])
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl MessageCache for NoopDataCache {
    type Error = NoopDataCacheError;

    async fn insert_messages<'a>(
        &self,
        _messages: impl IntoIterator<Item = &'a MessageLike> + SendUnlessWasm,
    ) -> Result<()> {
        Ok(())
    }

    async fn load_messages_targeting<'a>(
        &self,
        _conversation: &BareJid,
        _targets: &[Id],
        _newer_than: impl Into<Option<&'a Id>> + SendUnlessWasm,
        _include_targeted_messages: bool,
    ) -> Result<Vec<MessageLike>> {
        Ok(vec![])
    }

    async fn load_messages_before(
        &self,
        _conversation: &BareJid,
        _older_than: Option<&Id>,
        _max_count: u32,
    ) -> Result<Option<Page<MessageLike>>> {
        Ok(None)
    }

    async fn load_messages_after(
        &self,
        _conversation: &BareJid,
        _newer_than: &Id,
        _max_count: Option<u32>,
    ) -> Result<Vec<MessageLike>> {
        Ok(vec![])
    }

    async fn load_stanza_id(
        &self,
        _conversation: &BareJid,
        _message_id: &Id,
    ) -> Result<Option<prose_xmpp::stanza::message::stanza_id::Id>> {
        Ok(None)
    }

    async fn save_draft(&self, _conversation: &BareJid, _text: Option<&str>) -> Result<()> {
        Ok(())
    }

    async fn load_draft(&self, _conversation: &BareJid) -> Result<Option<String>> {
        Ok(None)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl DataCache for NoopDataCache {
    type Error = NoopDataCacheError;

    async fn delete_all(&self) -> Result<()> {
        Ok(())
    }

    async fn save_account_settings(&self, _settings: &AccountSettings) -> Result<()> {
        Ok(())
    }

    async fn load_account_settings(&self) -> Result<Option<AccountSettings>> {
        Ok(None)
    }
}
