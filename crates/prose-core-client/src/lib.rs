#[cfg(feature = "native-app")]
pub use avatar_cache::fs_avatar_cache::FsAvatarCache;
#[cfg(feature = "native-app")]
pub use bookmarks::{AccountBookmark, AccountBookmarksClient};
pub use client::{
    CachePolicy, Client, ClientBuilder, ClientDelegate, ClientEvent, ConnectionEvent,
};

pub mod avatar_cache;
mod client;
pub mod data_cache;
pub(crate) mod domain_ext;
pub mod types;

#[cfg(feature = "test-helpers")]
pub mod test;

#[cfg(feature = "native-app")]
mod bookmarks;
mod util;
