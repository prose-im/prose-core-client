// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[cfg(not(target_arch = "wasm32"))]
pub use avatar_cache::fs_avatar_cache::FsAvatarCache;
#[cfg(not(target_arch = "wasm32"))]
pub use bookmarks::{AccountBookmark, AccountBookmarksClient};
pub use client::{
    room, CachePolicy, Client, ClientBuilder, ClientDelegate, ClientEvent, ConnectionEvent,
};

pub mod avatar_cache;
mod client;
pub mod data_cache;
pub mod types;

#[cfg(feature = "test")]
pub mod test;

#[cfg(not(target_arch = "wasm32"))]
mod bookmarks;
mod util;
