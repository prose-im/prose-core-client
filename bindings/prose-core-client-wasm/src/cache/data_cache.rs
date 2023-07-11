use async_trait::async_trait;
use indexed_db_futures::prelude::*;
use indexed_db_futures::web_sys::DomException;
use indexed_db_futures::{IdbDatabase, IdbVersionChangeEvent};
use jid::BareJid;
use prose_core_client::types::roster::Item;
use prose_core_client::types::{AccountSettings, AvatarMetadata, MessageLike, Page};
use prose_core_client::{ContactsCache, DataCache, MessageCache};
use prose_domain::{Contact, UserProfile};
use prose_xmpp::stanza::avatar::ImageId;
use prose_xmpp::stanza::message::ChatState;
use prose_xmpp::stanza::{message, presence};
use thiserror::Error;
use wasm_bindgen::JsValue;

mod keys {
    pub const DB_NAME: &str = "ProseCache";

    pub const MESSAGES_STORE: &str = "messages";
}

#[derive(Error, Debug)]
pub enum IndexedDBDataCacheError {
    #[error("DomException {name} ({code}): {message}")]
    DomException {
        code: u16,
        name: String,
        message: String,
    },

    #[error(transparent)]
    JSON(#[from] serde_json::error::Error),
}

impl From<DomException> for IndexedDBDataCacheError {
    fn from(value: DomException) -> Self {
        IndexedDBDataCacheError::DomException {
            code: value.code(),
            name: value.name(),
            message: value.message(),
        }
    }
}

type Result<T, E = IndexedDBDataCacheError> = std::result::Result<T, E>;

pub struct IndexedDBDataCache {
    db: IdbDatabase,
}

impl IndexedDBDataCache {
    pub async fn new() -> Result<Self> {
        let mut db_req = IdbDatabase::open_u32(keys::DB_NAME, 1)?;

        db_req.set_on_upgrade_needed(Some(|evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            let old_version = evt.old_version() as u32;
            let db = evt.db();

            if old_version < 1 {
                db.create_object_store(keys::MESSAGES_STORE)?;
            }

            Ok(())
        }));

        let db = db_req.into_future().await?;

        Ok(IndexedDBDataCache { db })
    }
}

#[async_trait(? Send)]
impl ContactsCache for IndexedDBDataCache {
    type Error = IndexedDBDataCacheError;

    async fn has_valid_roster_items(&self) -> Result<bool> {
        Ok(false)
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

    async fn insert_presence(
        &self,
        _jid: &BareJid,
        _kind: Option<presence::Type>,
        _show: Option<presence::Show>,
        _status: Option<String>,
    ) -> Result<()> {
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

#[async_trait(? Send)]
impl MessageCache for IndexedDBDataCache {
    type Error = IndexedDBDataCacheError;

    async fn insert_messages<'a>(
        &self,
        messages: impl IntoIterator<Item = &'a MessageLike>,
    ) -> Result<()> {
        let tx = self
            .db
            .transaction_on_one_with_mode(keys::MESSAGES_STORE, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(keys::MESSAGES_STORE)?;

        for message in messages {
            store.put_key_val(
                &JsValue::from_str(message.id.as_ref()),
                &JsValue::from_serde(message)?,
            )?;
        }

        tx.await.into_result()?;
        Ok(())
    }

    async fn load_messages_targeting<'a>(
        &self,
        _conversation: &BareJid,
        targets: &[message::Id],
        _newer_than: impl Into<Option<&'a message::Id>>,
        _include_targeted_messages: bool,
    ) -> Result<Vec<MessageLike>> {
        Ok(vec![])
    }

    async fn load_messages_before(
        &self,
        _conversation: &BareJid,
        _older_than: Option<&message::Id>,
        _max_count: u32,
    ) -> Result<Option<Page<MessageLike>>> {
        Ok(None)
    }

    async fn load_messages_after(
        &self,
        _conversation: &BareJid,
        _newer_than: &message::Id,
        _max_count: Option<u32>,
    ) -> Result<Vec<MessageLike>> {
        Ok(vec![])
    }

    async fn load_stanza_id(
        &self,
        _conversation: &BareJid,
        _message_id: &message::Id,
    ) -> Result<Option<message::stanza_id::Id>> {
        Ok(None)
    }

    async fn save_draft(&self, _conversation: &BareJid, _text: Option<&str>) -> Result<()> {
        Ok(())
    }

    async fn load_draft(&self, _conversation: &BareJid) -> Result<Option<String>> {
        Ok(None)
    }
}

#[async_trait(? Send)]
impl DataCache for IndexedDBDataCache {
    type Error = IndexedDBDataCacheError;

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
