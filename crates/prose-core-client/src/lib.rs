#[cfg(not(target_arch = "wasm32"))]
pub use avatar_cache::fs_avatar_cache::FsAvatarCache;
#[cfg(not(target_arch = "wasm32"))]
pub use bookmarks::{AccountBookmark, AccountBookmarksClient};
pub use client::{
    CachePolicy, Client, ClientBuilder, ClientDelegate, ClientEvent, ConnectionEvent,
};

pub mod avatar_cache;
mod client;
pub mod data_cache;
pub(crate) mod domain_ext;
pub mod types;

#[cfg(feature = "test")]
pub mod test;

#[cfg(not(target_arch = "wasm32"))]
mod bookmarks;
mod util;
