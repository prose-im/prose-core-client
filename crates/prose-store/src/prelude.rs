pub use crate::{
    driver::Driver, store::Store, Database, IndexSpec, IndexedCollection, Query, QueryDirection,
    ReadTransaction, ReadableCollection, StoreError, UpgradeTransaction, WritableCollection,
    WriteTransaction,
};

#[cfg(target_arch = "wasm32")]
pub use crate::driver::indexed_db::{Error, IndexedDBDriver};
#[cfg(not(target_arch = "wasm32"))]
pub use crate::driver::sqlite::{Error, SqliteDriver};
