#[cfg(feature = "native-app")]
pub use bookmarks::{AccountBookmark, AccountBookmarksClient};
#[cfg(feature = "native-app")]
pub use cache::fs_avatar_cache::FsAvatarCache;
#[cfg(any(feature = "native-app", feature = "test-helpers"))]
pub use cache::sqlite_data_cache::SQLiteCache;
pub use cache::{ContactsCache, DataCache, MessageCache, NoopAvatarCache, NoopDataCache};
pub use client::{
    CachePolicy, Client, ClientBuilder, ClientDelegate, ClientEvent, ConnectionEvent,
};

mod cache;
mod client;
mod domain_ext;
pub mod types;

#[cfg(feature = "test-helpers")]
pub mod test_helpers;

#[cfg(feature = "native-app")]
mod bookmarks;
