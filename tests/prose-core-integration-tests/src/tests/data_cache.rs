use anyhow::Result;
use prose_core_client::data_cache::DataCache;
use prose_core_client::types::{AccountSettings, Availability};

#[cfg(target_arch = "wasm32")]
use prose_core_client::data_cache::indexed_db::IndexedDBDataCache;
#[cfg(not(target_arch = "wasm32"))]
use prose_core_client::data_cache::sqlite::{Connection, SQLiteCache};

#[cfg(not(target_arch = "wasm32"))]
use tokio::test as async_test;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test as async_test;

#[cfg(not(target_arch = "wasm32"))]
async fn cache() -> Result<SQLiteCache> {
    Ok(SQLiteCache::open_with_connection(
        Connection::open_in_memory()?,
    )?)
}

#[cfg(target_arch = "wasm32")]
async fn cache() -> Result<IndexedDBDataCache> {
    Ok(IndexedDBDataCache::new().await?)
}

#[async_test]
async fn test_save_and_load_account_settings() -> Result<()> {
    let cache = cache().await?;

    assert_eq!(cache.load_account_settings().await?, None);

    let settings = AccountSettings {
        availability: Availability::Away,
    };

    cache.save_account_settings(&settings).await?;
    assert_eq!(cache.load_account_settings().await?, Some(settings));

    Ok(())
}
