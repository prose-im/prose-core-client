// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use client::{Client, ClientDelegate};
pub use client_event::ClientEvent;
#[cfg(not(target_arch = "wasm32"))]
pub use util::account_bookmarks_client::{AccountBookmark, AccountBookmarksClient};

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

#[cfg(feature = "test")]
pub mod infra;
#[cfg(not(feature = "test"))]
pub(crate) mod infra;

pub(crate) mod util;
