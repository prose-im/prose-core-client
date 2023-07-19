use std::future::IntoFuture;
use std::str::FromStr;

use async_trait::async_trait;
use jid::BareJid;
use thiserror::Error;
use tracing::debug;

use indexed_db_futures::prelude::*;
use indexed_db_futures::web_sys::{DomException, IdbKeyRange};
use indexed_db_futures::{IdbDatabase, IdbVersionChangeEvent};
use prose_domain::{Availability, Contact, UserProfile};
use prose_xmpp::stanza::avatar::ImageId;
use prose_xmpp::stanza::message::ChatState;
use prose_xmpp::stanza::{message, presence};
use wasm_bindgen::JsValue;

use crate::data_cache::{ContactsCache, DataCache, MessageCache};
use crate::types::{roster, AccountSettings, AvatarMetadata, MessageLike, Page};

mod keys {
    pub const DB_NAME: &str = "ProseCache";

    pub const SETTINGS_STORE: &str = "settings";
    pub const MESSAGES_STORE: &str = "messages";
    pub const ROSTER_ITEMS_STORE: &str = "roster_item";
    pub const USER_PROFILE_STORE: &str = "user_profile";

    pub const TARGET_INDEX: &str = "target_idx";
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

    #[error(transparent)]
    JID(#[from] jid::JidParseError),

    #[error("Invalid DB Key")]
    InvalidDBKey,
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
        let mut db_req = IdbDatabase::open_u32(keys::DB_NAME, 2)?;

        db_req.set_on_upgrade_needed(Some(|evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            let old_version = evt.old_version() as u32;
            let db = evt.db();

            if old_version < 1 {
                db.create_object_store(keys::SETTINGS_STORE)?;
                db.create_object_store(keys::MESSAGES_STORE)?;
                db.create_object_store(keys::ROSTER_ITEMS_STORE)?;
                db.create_object_store(keys::USER_PROFILE_STORE)?;
            }

            if old_version < 2 {
                db.delete_object_store(keys::MESSAGES_STORE)?;

                let store = db.create_object_store(keys::MESSAGES_STORE)?;
                store.create_index_with_params(
                    keys::TARGET_INDEX,
                    &IdbKeyPath::str("target"),
                    &IdbIndexParameters::new().unique(false),
                )?;
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

    async fn insert_roster_items(&self, items: &[roster::Item]) -> Result<()> {
        debug!("Store {} roster items.", items.len());

        let tx = self.db.transaction_on_one_with_mode(
            keys::ROSTER_ITEMS_STORE,
            IdbTransactionMode::Readwrite,
        )?;
        let store = tx.object_store(keys::ROSTER_ITEMS_STORE)?;

        for item in items {
            store.put_key_val(
                &JsValue::from_str(&item.jid.to_string()),
                &JsValue::from_serde(item)?,
            )?;
        }

        tx.await.into_result()?;
        Ok(())
    }

    async fn insert_user_profile(&self, jid: &BareJid, profile: &UserProfile) -> Result<()> {
        debug!("Store profile for {}.", jid);

        let tx = self.db.transaction_on_one_with_mode(
            keys::USER_PROFILE_STORE,
            IdbTransactionMode::Readwrite,
        )?;
        let store = tx.object_store(keys::USER_PROFILE_STORE)?;

        store.put_key_val(
            &JsValue::from_str(&jid.to_string()),
            &JsValue::from_serde(profile)?,
        )?;

        tx.await.into_result()?;

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
        let tx = self.db.transaction_on_multi_with_mode(
            &[keys::USER_PROFILE_STORE, keys::ROSTER_ITEMS_STORE],
            IdbTransactionMode::Readonly,
        )?;

        let roster_items_store = tx.object_store(keys::ROSTER_ITEMS_STORE)?;
        let user_profile_store = tx.object_store(keys::USER_PROFILE_STORE)?;
        let jids = roster_items_store.get_all_keys()?.await?;
        let mut contacts = vec![];

        for jid in jids {
            let parsed_jid = BareJid::from_str(
                &jid.as_string()
                    .ok_or(IndexedDBDataCacheError::InvalidDBKey)?,
            )?;

            let Some(roster_item): Option<roster::Item> = roster_items_store
                .get(&jid)?
                .await?
                .map(|v| v.into_serde())
                .transpose()?
            else {
                continue;
            };

            let user_profile: Option<UserProfile> = user_profile_store
                .get(&jid)?
                .await?
                .map(|v| v.into_serde())
                .transpose()?;

            let full_name = user_profile.as_ref().and_then(|u| u.full_name.clone());
            let nickname = user_profile.as_ref().and_then(|u| u.nickname.clone());

            let contact = Contact {
                jid: parsed_jid.clone(),
                name: full_name.or(nickname).unwrap_or(parsed_jid.to_string()),
                avatar: None,
                availability: Availability::Unavailable,
                status: None,
                groups: roster_item.groups,
            };
            contacts.push((contact, None))
        }

        Ok(contacts)
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
        let tx = self
            .db
            .transaction_on_one_with_mode(keys::MESSAGES_STORE, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(keys::MESSAGES_STORE)?;
        let targetIdx = store.index(keys::TARGET_INDEX)?;

        for target in targets {
            let range = IdbKeyRange::only(&JsValue::from_str(target.as_ref()))
                .map_err(|_| IndexedDBDataCacheError::InvalidDBKey)?;
            let cursor = targetIdx.open_key_cursor_with_range_owned(range)?.await?;
        }

        // openRequest.onsuccess = (event) => {
        //     const db = event.target.result;
        //     const messagesStore = db.transaction('messages', 'readonly').objectStore('messages');
        //     const targetIndex = messagesStore.index('target');
        //
        //     const cursorRequest = targetIndex.openCursor(IDBKeyRange.only(searchValue));
        //
        //     cursorRequest.onsuccess = (event) => {
        //         const cursor = event.target.result;
        //         if (cursor) {
        //             // If the "target" key matches the search value, add it to the results array
        //             if (cursor.value.target === searchValue) {
        //                 results.push(cursor.value);
        //             }
        //
        //             // Continue searching
        //             cursor.continue();
        //         } else {
        //             // If the cursor is null, we have processed all records
        //             // Now let's check if the "id" key matches the search value
        //             messagesStore.get(searchValue).onsuccess = (event) => {
        //                 if (event.target.result) {
        //                     // Add the result to the results array if it matches the search value
        //                     results.push(event.target.result);
        //                 }
        //
        //                 console.log(results); // Do something with the fetched records
        //             };
        //         }
        //     };
        // };

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
