use std::collections::HashMap;
use std::sync::Mutex;

use anyhow::Result;
use indexed_db_futures::prelude::IdbOpenDbRequestLike;
use indexed_db_futures::web_sys::DomException;
use indexed_db_futures::{IdbDatabase, IdbVersionChangeEvent};
use jid::BareJid;
use wasm_bindgen::JsValue;

use prose_core_client::types::roster::Item;
use prose_core_client::types::{AccountSettings, AvatarMetadata, MessageLike, Page};
use prose_core_client::{ContactsCache, DataCache, MessageCache};
use prose_domain::{Contact, UserProfile};
use prose_xmpp::stanza::avatar::ImageId;
use prose_xmpp::stanza::message::ChatState;
use prose_xmpp::stanza::{message, presence};

pub struct InMemoryDataCache {
    db: IdbDatabase,
    messages: Mutex<HashMap<message::Id, MessageLike>>,
}

impl InMemoryDataCache {
    pub async fn new() -> Result<Self, DomException> {
        let mut db_req = IdbDatabase::open("ProseCache")?;
        db_req.set_on_upgrade_needed(Some(|evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            if !evt.db().object_store_names().any(|n| n == "ProseStore") {
                evt.db().create_object_store("ProseStore")?;
            }
            Ok(())
        }));

        let db = db_req.into_future().await?;

        Ok(InMemoryDataCache {
            db,
            messages: Mutex::new(HashMap::new()),
        })
    }
}

impl ContactsCache for InMemoryDataCache {
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
        _kind: Option<presence::Type>,
        _show: Option<presence::Show>,
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

impl MessageCache for InMemoryDataCache {
    fn insert_messages<'a>(
        &self,
        messages: impl IntoIterator<Item = &'a MessageLike>,
    ) -> Result<()> {
        let mut cached_messages = self.messages.lock().unwrap();

        for message in messages.into_iter() {
            cached_messages.insert(message.id.clone(), message.clone());
        }
        Ok(())
    }

    fn load_messages_targeting<'a>(
        &self,
        _conversation: &BareJid,
        targets: &[message::Id],
        _newer_than: impl Into<Option<&'a message::Id>>,
        _include_targeted_messages: bool,
    ) -> Result<Vec<MessageLike>> {
        let all_messages = self.messages.lock().unwrap();

        let mut messages = vec![];
        for id in targets {
            if let Some(message) = all_messages.get(id) {
                messages.push(message.clone())
            }
        }

        Ok(messages)
    }

    fn load_messages_before(
        &self,
        _conversation: &BareJid,
        _older_than: Option<&message::Id>,
        _max_count: u32,
    ) -> Result<Option<Page<MessageLike>>> {
        Ok(None)
    }

    fn load_messages_after(
        &self,
        _conversation: &BareJid,
        _newer_than: &message::Id,
        _max_count: Option<u32>,
    ) -> Result<Vec<MessageLike>> {
        Ok(vec![])
    }

    fn load_stanza_id(
        &self,
        _conversation: &BareJid,
        _message_id: &message::Id,
    ) -> Result<Option<message::stanza_id::Id>> {
        Ok(None)
    }

    fn save_draft(&self, _conversation: &BareJid, _text: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn load_draft(&self, _conversation: &BareJid) -> Result<Option<String>> {
        Ok(None)
    }
}

impl DataCache for InMemoryDataCache {
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
