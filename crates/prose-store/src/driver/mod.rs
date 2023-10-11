use crate::{Database, StoreError, UpgradeTransaction, VersionChangeEvent};
use async_trait::async_trait;
use prose_wasm_utils::SendUnlessWasm;

#[cfg(target_arch = "wasm32")]
pub mod indexed_db;
#[cfg(not(target_arch = "wasm32"))]
pub mod sqlite;

pub trait ReadMode {}
pub trait WriteMode {}

pub struct ReadOnly;
pub struct ReadWrite;

impl ReadMode for ReadOnly {}
impl ReadMode for ReadWrite {}
impl WriteMode for ReadWrite {}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
pub trait Driver: SendUnlessWasm + 'static {
    type Error: StoreError + Send + Sync;
    type UpgradeTransaction<'db>: UpgradeTransaction<'db, Error = Self::Error>;
    type Database: Database<Error = Self::Error>;

    async fn open<F>(self, version: u32, update_handler: F) -> Result<Self::Database, Self::Error>
    where
        F: Fn(&VersionChangeEvent<Self::UpgradeTransaction<'_>>) -> Result<(), Self::Error>
            + Send
            + 'static;
}
