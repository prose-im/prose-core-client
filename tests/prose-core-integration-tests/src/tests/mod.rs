// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
pub use tokio::test as async_test;

use prose_core_client::infra::platform_dependencies::open_store;
use prose_store::prelude::*;
#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_test::wasm_bindgen_test as async_test;

mod account_settings_repository;
#[cfg(not(target_arch = "wasm32"))]
mod client;
mod client_omemo;
mod contacts_repository;
mod drafts_repository;
#[cfg(not(target_arch = "wasm32"))]
mod helpers;
mod messages_repository;
mod user_info_repository;

#[cfg(target_arch = "wasm32")]
type PlatformDriver = IndexedDBDriver;
#[cfg(not(target_arch = "wasm32"))]
type PlatformDriver = SqliteDriver;

#[cfg(target_arch = "wasm32")]
pub fn platform_driver(name: impl AsRef<str>) -> IndexedDBDriver {
    IndexedDBDriver::new(name)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn platform_driver(_name: impl AsRef<str>) -> SqliteDriver {
    let path = tempfile::tempdir().unwrap().path().join("test.sqlite");
    let parent = path.parent().unwrap();
    std::fs::create_dir_all(parent).unwrap();
    println!("Opening DB at {:?}", path);
    SqliteDriver::new(path)
}

async fn store() -> Result<Store<PlatformDriver>> {
    let driver = platform_driver(
        std::path::Path::new(file!())
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap(),
    );

    let store = open_store(driver).await?;
    store.truncate_all_collections().await?;
    Ok(store)
}
