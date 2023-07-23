use async_trait::async_trait;
#[cfg(feature = "test-helpers")]
use auto_impl::auto_impl;
use chrono::{DateTime, Utc};
use jid::BareJid;

use prose_domain::Contact;
use prose_xmpp::stanza::message::{stanza_id, ChatState};
use prose_xmpp::stanza::{avatar, message};
use prose_xmpp::{SendUnlessWasm, SyncUnlessWasm};

use crate::types::{
    roster, AccountSettings, AvatarMetadata, MessageLike, Page, Presence, UserProfile,
};

#[cfg_attr(feature = "test-helpers", auto_impl(Arc))]
#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
pub trait DataCache: ContactsCache + MessageCache + SendUnlessWasm + SyncUnlessWasm {
    type Error: std::error::Error + Send + Sync;

    async fn delete_all(&self) -> Result<(), <Self as DataCache>::Error>;

    async fn save_account_settings(
        &self,
        settings: &AccountSettings,
    ) -> Result<(), <Self as DataCache>::Error>;
    async fn load_account_settings(
        &self,
    ) -> Result<Option<AccountSettings>, <Self as DataCache>::Error>;
}

#[cfg_attr(feature = "test-helpers", auto_impl(Arc))]
#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
pub trait ContactsCache {
    type Error: std::error::Error + Send + Sync;

    async fn set_roster_update_time(&self, timestamp: &DateTime<Utc>) -> Result<(), Self::Error>;
    async fn roster_update_time(&self) -> Result<Option<DateTime<Utc>>, Self::Error>;

    async fn insert_roster_items(&self, items: &[roster::Item]) -> Result<(), Self::Error>;

    async fn insert_user_profile(
        &self,
        jid: &BareJid,
        profile: &UserProfile,
    ) -> Result<(), Self::Error>;
    async fn load_user_profile(&self, jid: &BareJid) -> Result<Option<UserProfile>, Self::Error>;
    async fn delete_user_profile(&self, jid: &BareJid) -> Result<(), Self::Error>;

    async fn insert_avatar_metadata(
        &self,
        jid: &BareJid,
        metadata: &AvatarMetadata,
    ) -> Result<(), Self::Error>;
    async fn load_avatar_metadata(
        &self,
        jid: &BareJid,
    ) -> Result<Option<AvatarMetadata>, Self::Error>;

    async fn insert_presence(&self, jid: &BareJid, presence: &Presence) -> Result<(), Self::Error>;

    async fn insert_chat_state(
        &self,
        jid: &BareJid,
        chat_state: &ChatState,
    ) -> Result<(), Self::Error>;
    async fn load_chat_state(&self, jid: &BareJid) -> Result<Option<ChatState>, Self::Error>;

    async fn load_contacts(&self) -> Result<Vec<(Contact, Option<avatar::ImageId>)>, Self::Error>;
}

#[cfg_attr(feature = "test-helpers", auto_impl(Arc))]
#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
pub trait MessageCache {
    type Error: std::error::Error + Send + Sync;

    async fn insert_messages<'a>(
        &self,
        messages: impl IntoIterator<Item = &'a MessageLike> + SendUnlessWasm,
    ) -> Result<(), Self::Error>;

    /// Loads all `MessageLike` objects from the cache that have a `target` contained in `targets`.
    /// Returns them ordered by `timestamp` in ascending order.
    ///
    /// # Arguments
    ///
    /// * `conversation`: The BareJid of the conversation to search in.
    /// * `targets`: The IDs of messages to be modified by the returned `MessageLike` objects.
    /// * `newer_than`: Load only `MessageLike` objects newer than the given message id.
    /// * `include_targeted_messages`: Whether to include the targeted messages as identified
    ///    by `targets` in the result.
    async fn load_messages_targeting<'a>(
        &self,
        conversation: &BareJid,
        targets: &[message::Id],
        newer_than: impl Into<Option<&'a message::Id>> + SendUnlessWasm,
        include_targeted_messages: bool,
    ) -> Result<Vec<MessageLike>, Self::Error>;

    /// Loads a page of `MessageLike` objects up to `max_count` items. Returns `None` if there are
    /// no objects in cache older than `older_than`. Returns an empty `Page` if the first page is
    /// cached but `older_than` is older than the first item. The items in `Page` are sorted in
    /// ascending order by their timestamp (higher index = newer message).
    ///
    /// # Arguments
    ///
    /// * `conversation`: The BareJid of the conversation to search in.
    /// * `older_than`: Load only `MessageLike` objects older than the given message id.
    /// * `max_count`: Load only up until `max_count` items.
    async fn load_messages_before(
        &self,
        conversation: &BareJid,
        older_than: Option<&message::Id>,
        max_count: u32,
    ) -> Result<Option<Page<MessageLike>>, Self::Error>;

    async fn load_messages_after(
        &self,
        conversation: &BareJid,
        newer_than: &message::Id,
        max_count: Option<u32>,
    ) -> Result<Vec<MessageLike>, Self::Error>;

    async fn load_stanza_id(
        &self,
        conversation: &BareJid,
        message_id: &message::Id,
    ) -> Result<Option<stanza_id::Id>, Self::Error>;

    async fn save_draft(
        &self,
        conversation: &BareJid,
        text: Option<&str>,
    ) -> Result<(), Self::Error>;
    async fn load_draft(&self, conversation: &BareJid) -> Result<Option<String>, Self::Error>;
}