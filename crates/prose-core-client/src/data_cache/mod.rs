// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use data_cache::{AccountCache, ContactsCache, DataCache, MessageCache};
pub use noop_data_cache::NoopDataCache;

mod data_cache;
mod noop_data_cache;

pub mod indexed_db;
