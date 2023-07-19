use async_trait::async_trait;
use gloo_utils::format::JsValueSerdeExt;
use indexed_db_futures::prelude::*;
use indexed_db_futures::web_sys::DomException;
use indexed_db_futures::{IdbDatabase, IdbVersionChangeEvent};
use thiserror::Error;
use wasm_bindgen::JsValue;

use crate::data_cache::DataCache;
use crate::types::AccountSettings;

pub(super) mod keys {
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

pub(super) type Result<T, E = IndexedDBDataCacheError> = std::result::Result<T, E>;

pub struct IndexedDBDataCache {
    pub(super) db: IdbDatabase,
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
impl DataCache for IndexedDBDataCache {
    type Error = IndexedDBDataCacheError;

    async fn delete_all(&self) -> Result<()> {
        Ok(())
    }

    async fn save_account_settings(&self, settings: &AccountSettings) -> Result<()> {
        let tx = self
            .db
            .transaction_on_one_with_mode(keys::SETTINGS_STORE, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(keys::SETTINGS_STORE)?;

        store.put_key_val(
            &JsValue::from_str("settings"),
            &JsValue::from_serde(settings)?,
        )?;

        tx.await.into_result()?;

        Ok(())
    }

    async fn load_account_settings(&self) -> Result<Option<AccountSettings>> {
        let tx = self
            .db
            .transaction_on_one_with_mode(keys::SETTINGS_STORE, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(keys::SETTINGS_STORE)?;

        let settings: Option<AccountSettings> = store
            .get(&JsValue::from_str("settings"))?
            .await?
            .map(|s| s.into_serde())
            .transpose()?;

        Ok(settings)
    }
}
