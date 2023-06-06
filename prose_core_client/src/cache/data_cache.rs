use jid::BareJid;

#[cfg(feature = "test-helpers")]
use auto_impl::auto_impl;

use prose_core_domain::Contact;
use prose_core_lib::modules::profile::avatar::ImageId;
use prose_core_lib::stanza::message::ChatState;
use prose_core_lib::stanza::{message, presence};

use crate::types::{AccountSettings, AvatarMetadata, MessageLike, Page, RosterItem, UserProfile};

#[cfg_attr(feature = "test-helpers", auto_impl(Arc))]
pub trait DataCache: ContactsCache + MessageCache + Send + Sync {
    fn delete_all(&self) -> anyhow::Result<()>;

    fn save_account_settings(&self, settings: &AccountSettings) -> anyhow::Result<()>;
    fn load_account_settings(&self) -> anyhow::Result<Option<AccountSettings>>;
}

#[cfg_attr(feature = "test-helpers", auto_impl(Arc))]
pub trait ContactsCache {
    fn has_valid_roster_items(&self) -> anyhow::Result<bool>;
    fn insert_roster_items(&self, items: &[RosterItem]) -> anyhow::Result<()>;

    fn insert_user_profile(&self, jid: &BareJid, profile: &UserProfile) -> anyhow::Result<()>;
    fn load_user_profile(&self, jid: &BareJid) -> anyhow::Result<Option<UserProfile>>;
    fn delete_user_profile(&self, jid: &BareJid) -> anyhow::Result<()>;

    fn insert_avatar_metadata(
        &self,
        jid: &BareJid,
        metadata: &AvatarMetadata,
    ) -> anyhow::Result<()>;
    fn load_avatar_metadata(&self, jid: &BareJid) -> anyhow::Result<Option<AvatarMetadata>>;

    fn insert_presence(
        &self,
        jid: &BareJid,
        kind: Option<presence::Kind>,
        show: Option<presence::Show>,
        status: Option<String>,
    ) -> anyhow::Result<()>;

    fn insert_chat_state(&self, jid: &BareJid, chat_state: &ChatState) -> anyhow::Result<()>;
    fn load_chat_state(&self, jid: &BareJid) -> anyhow::Result<Option<ChatState>>;

    fn load_contacts(&self) -> anyhow::Result<Vec<(Contact, Option<ImageId>)>>;
}

#[cfg_attr(feature = "test-helpers", auto_impl(Arc))]
pub trait MessageCache {
    fn insert_messages<'a>(
        &self,
        messages: impl IntoIterator<Item = &'a MessageLike>,
    ) -> anyhow::Result<()>
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
    ) -> anyhow::Result<Vec<MessageLike>>;

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
    ) -> anyhow::Result<Option<Page<MessageLike>>>;

    fn load_messages_after(
        &self,
        conversation: &BareJid,
        newer_than: &message::Id,
        max_count: Option<u32>,
    ) -> anyhow::Result<Vec<MessageLike>>;

    fn load_stanza_id(
        &self,
        conversation: &BareJid,
        message_id: &message::Id,
    ) -> anyhow::Result<Option<message::StanzaId>>;

    fn save_draft(&self, conversation: &BareJid, text: Option<&str>) -> anyhow::Result<()>;
    fn load_draft(&self, conversation: &BareJid) -> anyhow::Result<Option<String>>;
}
