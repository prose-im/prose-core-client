// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use app::{dtos, services};
pub use client::{Client, ClientDelegate};
pub use client_event::{ClientEvent, ConnectionEvent, RoomEventType};
pub use infra::platform_dependencies::open_store;
#[cfg(target_arch = "wasm32")]
pub use prose_store::prelude::IndexedDBDriver;
#[cfg(not(target_arch = "wasm32"))]
pub use prose_store::prelude::SqliteDriver;
#[cfg(not(target_arch = "wasm32"))]
pub use util::account_bookmarks_client::{AccountBookmark, AccountBookmarksClient};

#[cfg(target_arch = "wasm32")]
pub use crate::infra::avatars::StoreAvatarCache;

#[cfg(feature = "test")]
pub mod test;

pub mod app;
mod client;
mod client_builder;
mod client_event;

#[cfg(feature = "test")]
pub mod domain;
#[cfg(not(feature = "test"))]
pub(crate) mod domain;

pub mod infra;

pub(crate) mod util;
