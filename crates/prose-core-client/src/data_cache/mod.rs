pub use data_cache::{AccountCache, ContactsCache, DataCache, MessageCache};
pub use noop_data_cache::NoopDataCache;

mod data_cache;
mod noop_data_cache;

#[cfg(target_arch = "wasm32")]
pub mod indexed_db;
#[cfg(any(not(target_arch = "wasm32"), feature = "test"))]
pub mod sqlite;
