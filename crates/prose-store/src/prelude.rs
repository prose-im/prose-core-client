pub use crate::{
    driver::Driver,
    repository::{Entity, Repository},
    store::Store,
    upsert, Database, IndexSpec, IndexedCollection, KeyType, Query, QueryDirection,
    ReadTransaction, ReadableCollection, StoreError, UpgradeTransaction, WritableCollection,
    WriteTransaction,
};
pub use prose_proc_macros::entity;

#[cfg(target_arch = "wasm32")]
pub use crate::driver::indexed_db::{Error, IndexedDBDriver};
#[cfg(not(target_arch = "wasm32"))]
pub use crate::driver::sqlite::{Error, SqliteDriver};

#[cfg(target_arch = "wasm32")]
pub use IndexedDBDriver as PlatformDriver;
#[cfg(not(target_arch = "wasm32"))]
pub use SqliteDriver as PlatformDriver;
