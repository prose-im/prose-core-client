pub use crate::{
    define_entity,
    driver::Driver,
    repository::{Entity, Repository},
    store::Store,
    upsert, Database, IndexSpec, IndexedCollection, KeyType, Query, QueryDirection, RawKey,
    ReadTransaction, ReadableCollection, StoreError, UpgradeTransaction, WritableCollection,
    WriteTransaction,
};

#[cfg(target_arch = "wasm32")]
pub use crate::driver::indexed_db::{Error, IndexedDBDriver};
#[cfg(not(target_arch = "wasm32"))]
pub use crate::driver::sqlite::{Error, SqliteDriver};

#[cfg(target_arch = "wasm32")]
pub use IndexedDBDriver as PlatformDriver;
#[cfg(not(target_arch = "wasm32"))]
pub use SqliteDriver as PlatformDriver;

#[cfg(target_arch = "wasm32")]
pub use crate::driver::indexed_db::Error as DriverError;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::driver::sqlite::Error as DriverError;
