// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::data_cache::data_cache::AccountCache;
use async_trait::async_trait;
use prose_store::prelude::*;

use crate::data_cache::DataCache;
use crate::types::AccountSettings;

pub(super) mod keys {
    #[cfg(target_arch = "wasm32")]
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
        pub const ID_INDEX: &str = "id";
        pub const TARGET_INDEX: &str = "target";
        pub const TIMESTAMP_INDEX: &str = "timestamp";
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("{msg}")]
    Store { msg: String },
    #[error("Invalid message id")]
    InvalidMessageId,
}

impl<T: StoreError> From<T> for CacheError {
    fn from(value: T) -> Self {
        Self::Store {
            msg: value.to_string(),
        }
    }
}

pub struct IndexedDBDataCache<D: Driver> {
    pub(super) db: Store<D>,
}

#[cfg(target_arch = "wasm32")]
pub type PlatformCache = IndexedDBDataCache<IndexedDBDriver>;
#[cfg(not(target_arch = "wasm32"))]
pub type PlatformCache = IndexedDBDataCache<SqliteDriver>;

#[cfg(target_arch = "wasm32")]
impl PlatformCache {
    pub async fn new() -> Result<Self, CacheError> {
        Ok(Self::new_with_driver(IndexedDBDriver::new(keys::DB_NAME)).await?)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PlatformCache {
    pub async fn open(path: impl Into<std::path::PathBuf>) -> Result<Self, CacheError> {
        Ok(Self::new_with_driver(SqliteDriver::new(path.into().join("db.sqlite3"))).await?)
    }

    #[cfg(feature = "test")]
    pub async fn temporary_cache() -> Result<Self, CacheError> {
        let path = tempfile::tempdir().unwrap().path().join("test.sqlite");
        let parent = path.parent().unwrap();
        std::fs::create_dir_all(parent).unwrap();
        println!("Opening DB at {:?}", path);

        let cache = PlatformCache::open(path).await?;
        cache.delete_all().await?;
        Ok(cache)
    }
}

impl<D: Driver> IndexedDBDataCache<D> {
    pub async fn new_with_driver(driver: D) -> Result<Self, CacheError> {
        let db = Store::open(driver, 4, |event| {
            let old_version = event.old_version;
            let tx = &event.tx;

            if old_version < 1 {
                tx.create_collection(keys::AVATAR_METADATA_STORE)?;
                tx.create_collection(keys::CHAT_STATE_STORE)?;
                tx.create_collection(keys::DRAFTS_STORE)?;
                tx.create_collection(keys::PRESENCE_STORE)?;
                tx.create_collection(keys::ROSTER_ITEMS_STORE)?;
                tx.create_collection(keys::SETTINGS_STORE)?;
                tx.create_collection(keys::USER_PROFILE_STORE)?;

                let messages = tx.create_collection(keys::MESSAGES_STORE)?;

                messages.add_index(
                    IndexSpec::builder(keys::messages::ID_INDEX)
                        .unique()
                        .build(),
                )?;
                messages.add_index(IndexSpec::builder(keys::messages::TARGET_INDEX).build())?;
                messages.add_index(IndexSpec::builder(keys::messages::TIMESTAMP_INDEX).build())?;
            }

            if old_version < 2 {
                tx.create_collection(keys::USER_ACTIVITY_STORE)?;
            }

            if old_version < 3 {
                tx.create_collection(keys::AVATAR_STORE)?;
            }

            if old_version < 4 {
                tx.delete_collection(keys::ROSTER_ITEMS_STORE)?;
                tx.create_collection(keys::ROSTER_ITEMS_STORE)?;
            }

            Ok(())
        })
        .await?;

        // Clear (outdated) presence entries from our last session.
        // db.clear_stores(&[keys::PRESENCE_STORE, keys::CHAT_STATE_STORE])
        //     .await?;

        Ok(IndexedDBDataCache { db })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl<D: Driver> AccountCache for IndexedDBDataCache<D> {
    type Error = CacheError;

    async fn delete_all(&self) -> Result<(), Self::Error> {
        self.db.truncate_all_collections().await?;
        Ok(())
    }

    async fn save_account_settings(&self, settings: &AccountSettings) -> Result<(), Self::Error> {
        self.db
            .transaction_for_reading_and_writing(&[keys::SETTINGS_STORE])
            .await?
            .writeable_collection(keys::SETTINGS_STORE)?
            .put(keys::settings::ACCOUNT, settings)?;
        Ok(())
    }

    async fn load_account_settings(&self) -> Result<Option<AccountSettings>, Self::Error> {
        let settings = self
            .db
            .transaction_for_reading(&[keys::SETTINGS_STORE])
            .await?
            .readable_collection(keys::SETTINGS_STORE)?
            .get(keys::settings::ACCOUNT)
            .await?;
        Ok(settings)
    }
}

impl<D: Driver> DataCache for IndexedDBDataCache<D> {}
