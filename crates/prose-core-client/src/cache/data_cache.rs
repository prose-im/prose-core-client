use anyhow::Result;
#[cfg(feature = "test-helpers")]
use auto_impl::auto_impl;
use jid::BareJid;
use prose_domain::Contact;
use prose_xmpp::stanza::message::{stanza_id, ChatState};
use prose_xmpp::stanza::{avatar, message};
use prose_xmpp::{SendUnlessWasm, SyncUnlessWasm};
use xmpp_parsers::presence;

use crate::types::{roster, AccountSettings, AvatarMetadata, MessageLike, Page, UserProfile};

#[cfg_attr(feature = "test-helpers", auto_impl(Arc))]
pub trait DataCache: ContactsCache + MessageCache + SendUnlessWasm + SyncUnlessWasm {
    fn delete_all(&self) -> Result<()>;

    fn save_account_settings(&self, settings: &AccountSettings) -> Result<()>;
    fn load_account_settings(&self) -> Result<Option<AccountSettings>>;
}

#[cfg_attr(feature = "test-helpers", auto_impl(Arc))]
pub trait ContactsCache {
    fn has_valid_roster_items(&self) -> Result<bool>;
    fn insert_roster_items(&self, items: &[roster::Item]) -> Result<()>;

    fn insert_user_profile(&self, jid: &BareJid, profile: &UserProfile) -> Result<()>;
    fn load_user_profile(&self, jid: &BareJid) -> Result<Option<UserProfile>>;
    fn delete_user_profile(&self, jid: &BareJid) -> Result<()>;

    fn insert_avatar_metadata(&self, jid: &BareJid, metadata: &AvatarMetadata) -> Result<()>;
    fn load_avatar_metadata(&self, jid: &BareJid) -> Result<Option<AvatarMetadata>>;

    fn insert_presence(
        &self,
        jid: &BareJid,
        kind: Option<presence::Type>,
        show: Option<presence::Show>,
        status: Option<String>,
    ) -> Result<()>;

    fn insert_chat_state(&self, jid: &BareJid, chat_state: &ChatState) -> Result<()>;
    fn load_chat_state(&self, jid: &BareJid) -> Result<Option<ChatState>>;

    fn load_contacts(&self) -> Result<Vec<(Contact, Option<avatar::ImageId>)>>;
}

#[cfg_attr(feature = "test-helpers", auto_impl(Arc))]
pub trait MessageCache {
    fn insert_messages<'a>(
        &self,
        messages: impl IntoIterator<Item = &'a MessageLike>,
    ) -> Result<()>
    where
        Self: Sized;

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
    fn load_messages_targeting<'a>(
        &self,
        conversation: &BareJid,
        targets: &[message::Id],
        newer_than: impl Into<Option<&'a message::Id>>,
        include_targeted_messages: bool,
    ) -> Result<Vec<MessageLike>>;

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
    fn load_messages_before(
        &self,
        conversation: &BareJid,
        older_than: Option<&message::Id>,
        max_count: u32,
    ) -> Result<Option<Page<MessageLike>>>;

    fn load_messages_after(
        &self,
        conversation: &BareJid,
        newer_than: &message::Id,
        max_count: Option<u32>,
    ) -> Result<Vec<MessageLike>>;

    fn load_stanza_id(
        &self,
        conversation: &BareJid,
        message_id: &message::Id,
    ) -> Result<Option<stanza_id::Id>>;

    fn save_draft(&self, conversation: &BareJid, text: Option<&str>) -> Result<()>;
    fn load_draft(&self, conversation: &BareJid) -> Result<Option<String>>;
}
