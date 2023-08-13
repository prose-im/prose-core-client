// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::data_cache::data_cache::AccountCache;
use async_trait::async_trait;
use indexed_db_futures::prelude::*;
use indexed_db_futures::web_sys::DomException;
use indexed_db_futures::{IdbDatabase, IdbVersionChangeEvent};
use thiserror::Error;
use wasm_bindgen::JsValue;

use crate::data_cache::indexed_db::idb_database_ext::IdbDatabaseExt;
use crate::data_cache::DataCache;
use crate::types::AccountSettings;

pub(super) mod keys {
    pub const DB_NAME: &str = "ProseCache";

    pub const SETTINGS_STORE: &str = "settings";
    pub const MESSAGES_STORE: &str = "messages";
    pub const ROSTER_ITEMS_STORE: &str = "roster_item";
    pub const USER_PROFILE_STORE: &str = "user_profile";
    pub const PRESENCE_STORE: &str = "presence";
    pub const AVATAR_METADATA_STORE: &str = "avatar_metadata";
    pub const CHAT_STATE_STORE: &str = "chat_state";
    pub const DRAFTS_STORE: &str = "drafts";
    pub const USER_ACTIVITY_STORE: &str = "user_activity";
    pub const AVATAR_STORE: &str = "avatar";

    pub mod settings {
        pub const ACCOUNT: &str = "account";
        pub const ROSTER_UPDATE: &str = "roster_updated";
    }

    pub mod messages {
        pub const ID_INDEX: &str = "id_idx";
        pub const TARGET_INDEX: &str = "target_idx";
        pub const TIMESTAMP_INDEX: &str = "timestamp_idx";
    }
}

#[derive(Error, Debug)]
pub enum IndexedDBDataCacheError {
    #[error("DomException {name}: {message}")]
    DomException { name: String, message: String },

    #[error(transparent)]
    JSON(#[from] serde_json::error::Error),

    #[error(transparent)]
    JID(#[from] jid::Error),

    #[error("Invalid DB Key")]
    InvalidDBKey,

    #[error("Invalid MessageId")]
    InvalidMessageId,
}

impl From<DomException> for IndexedDBDataCacheError {
    fn from(value: DomException) -> Self {
        IndexedDBDataCacheError::DomException {
            name: value.name(),
            message: value.message(),
        }
    }
}

pub(super) type Result<T, E = IndexedDBDataCacheError> = std::result::Result<T, E>;

pub struct IndexedDBDataCache {
    pub(super) db: IdbDatabase,
}

impl IndexedDBDataCache {
    pub async fn new() -> Result<Self> {
        let mut db_req = IdbDatabase::open_u32(keys::DB_NAME, 3)?;

        db_req.set_on_upgrade_needed(Some(|evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            let old_version = evt.old_version() as u32;
            let db = evt.db();

            if old_version < 1 {
                db.create_object_store(keys::AVATAR_METADATA_STORE)?;
                db.create_object_store(keys::CHAT_STATE_STORE)?;
                db.create_object_store(keys::DRAFTS_STORE)?;
                db.create_object_store(keys::PRESENCE_STORE)?;
                db.create_object_store(keys::ROSTER_ITEMS_STORE)?;
                db.create_object_store(keys::SETTINGS_STORE)?;
                db.create_object_store(keys::USER_PROFILE_STORE)?;

                let store = db.create_object_store_with_params(
                    keys::MESSAGES_STORE,
                    &IdbObjectStoreParameters::new()
                        .auto_increment(true)
                        .key_path(Some(&IdbKeyPath::str("rowId"))),
                )?;
                store.create_index_with_params(
                    keys::messages::ID_INDEX,
                    &IdbKeyPath::str("id"),
                    &IdbIndexParameters::new().unique(true),
                )?;
                store.create_index_with_params(
                    keys::messages::TARGET_INDEX,
                    &IdbKeyPath::str("target"),
                    &IdbIndexParameters::new().unique(false),
                )?;
                store.create_index_with_params(
                    keys::messages::TIMESTAMP_INDEX,
                    &IdbKeyPath::str("timestamp"),
                    &IdbIndexParameters::new().unique(false),
                )?;
            }

            if old_version < 2 {
                db.create_object_store(keys::USER_ACTIVITY_STORE)?;
            }

            if old_version < 3 {
                db.create_object_store(keys::AVATAR_STORE)?;
            }

            Ok(())
        }));

        let db = db_req.into_future().await?;

        // Clear (outdated) presence entries from our last session.
        db.clear_stores(&[keys::PRESENCE_STORE, keys::CHAT_STATE_STORE])
            .await?;

        Ok(IndexedDBDataCache { db })
    }
}

#[async_trait(? Send)]
impl AccountCache for IndexedDBDataCache {
    type Error = IndexedDBDataCacheError;

    async fn delete_all(&self) -> Result<()> {
        self.db
            .clear_stores(&[
                keys::AVATAR_METADATA_STORE,
                keys::CHAT_STATE_STORE,
                keys::DRAFTS_STORE,
                keys::MESSAGES_STORE,
                keys::PRESENCE_STORE,
                keys::ROSTER_ITEMS_STORE,
                keys::SETTINGS_STORE,
                keys::USER_PROFILE_STORE,
            ])
            .await
    }

    async fn save_account_settings(&self, settings: &AccountSettings) -> Result<()> {
        self.db
            .set_value(keys::SETTINGS_STORE, keys::settings::ACCOUNT, settings)
            .await
    }

    async fn load_account_settings(&self) -> Result<Option<AccountSettings>> {
        self.db
            .get_value(keys::SETTINGS_STORE, keys::settings::ACCOUNT)
            .await
    }
}

impl DataCache for IndexedDBDataCache {}
