pub use data_cache::{AccountCache, ContactsCache, DataCache, MessageCache};
pub use noop_data_cache::NoopDataCache;

mod data_cache;
mod noop_data_cache;

#[cfg(feature = "js")]
pub mod indexed_db;
#[cfg(any(feature = "native-app", feature = "test-helpers"))]
pub mod sqlite;
