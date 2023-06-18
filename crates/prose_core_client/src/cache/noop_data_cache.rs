use anyhow::Result;
use jid::BareJid;
use prose_core_domain::{Contact, UserProfile};
use prose_core_lib::stanza::avatar::ImageId;
use prose_core_lib::stanza::message::{ChatState, Id};
use xmpp_parsers::presence::{Show, Type};

use crate::cache::ContactsCache;
use crate::types::roster::Item;
use crate::types::{AccountSettings, AvatarMetadata, MessageLike, Page};
use crate::{DataCache, MessageCache};

pub struct NoopDataCache {}

impl Default for NoopDataCache {
    fn default() -> Self {
        NoopDataCache {}
    }
}

impl ContactsCache for NoopDataCache {
    fn has_valid_roster_items(&self) -> Result<bool> {
        Ok(false)
    }

    fn insert_roster_items(&self, _items: &[Item]) -> Result<()> {
        Ok(())
    }

    fn insert_user_profile(&self, _jid: &BareJid, _profile: &UserProfile) -> Result<()> {
        Ok(())
    }

    fn load_user_profile(&self, _jid: &BareJid) -> Result<Option<UserProfile>> {
        Ok(None)
    }

    fn delete_user_profile(&self, _jid: &BareJid) -> Result<()> {
        Ok(())
    }

    fn insert_avatar_metadata(&self, _jid: &BareJid, _metadata: &AvatarMetadata) -> Result<()> {
        Ok(())
    }

    fn load_avatar_metadata(&self, _jid: &BareJid) -> Result<Option<AvatarMetadata>> {
        Ok(None)
    }

    fn insert_presence(
        &self,
        _jid: &BareJid,
        _kind: Option<Type>,
        _show: Option<Show>,
        _status: Option<String>,
    ) -> Result<()> {
        Ok(())
    }

    fn insert_chat_state(&self, _jid: &BareJid, _chat_state: &ChatState) -> Result<()> {
        Ok(())
    }

    fn load_chat_state(&self, _jid: &BareJid) -> Result<Option<ChatState>> {
        Ok(None)
    }

    fn load_contacts(&self) -> Result<Vec<(Contact, Option<ImageId>)>> {
        Ok(vec![])
    }
}

impl MessageCache for NoopDataCache {
    fn insert_messages<'a>(
        &self,
        _messages: impl IntoIterator<Item = &'a MessageLike>,
    ) -> Result<()>
    where
        Self: Sized,
    {
        Ok(())
    }

    fn load_messages_targeting<'a>(
        &self,
        _conversation: &BareJid,
        _targets: &[Id],
        _newer_than: impl Into<Option<&'a Id>>,
        _include_targeted_messages: bool,
    ) -> Result<Vec<MessageLike>> {
        Ok(vec![])
    }

    fn load_messages_before(
        &self,
        _conversation: &BareJid,
        _older_than: Option<&Id>,
        _max_count: u32,
    ) -> Result<Option<Page<MessageLike>>> {
        Ok(None)
    }

    fn load_messages_after(
        &self,
        _conversation: &BareJid,
        _newer_than: &Id,
        _max_count: Option<u32>,
    ) -> Result<Vec<MessageLike>> {
        Ok(vec![])
    }

    fn load_stanza_id(
        &self,
        _conversation: &BareJid,
        _message_id: &Id,
    ) -> Result<Option<prose_core_lib::stanza::message::stanza_id::Id>> {
        Ok(None)
    }

    fn save_draft(&self, _conversation: &BareJid, _text: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn load_draft(&self, _conversation: &BareJid) -> Result<Option<String>> {
        Ok(None)
    }
}

impl DataCache for NoopDataCache {
    fn delete_all(&self) -> Result<()> {
        Ok(())
    }

    fn save_account_settings(&self, _settings: &AccountSettings) -> Result<()> {
        Ok(())
    }

    fn load_account_settings(&self) -> Result<Option<AccountSettings>> {
        Ok(None)
    }
}
