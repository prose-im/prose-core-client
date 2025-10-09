// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

extern crate core;

pub use secrecy::SecretString;

pub use app::deps::{DynEncryptionKeysRepository, DynSessionRepository};
pub use app::{dtos, services};
pub use client::{Client, ClientDelegate};
pub use client_event::{ClientEvent, ClientRoomEventType, ConnectionEvent};
#[cfg(not(target_arch = "wasm32"))]
pub use domain::encryption::services::impls::signal_native::SignalServiceHandle;
pub use domain::encryption::services::EncryptionService;
pub use infra::platform_dependencies::open_store;
#[cfg(feature = "test")]
pub use infra::xmpp::event_parser::parse_xmpp_event;
pub use prose_store::prelude::{Error as StoreError, PlatformDriver, Store};
#[cfg(not(target_arch = "wasm32"))]
pub use util::account_bookmarks_client::{AccountBookmark, AccountBookmarksClient};
#[cfg(feature = "debug")]
pub use util::RandomDelayProxyTransformer;

#[cfg(target_arch = "wasm32")]
pub use crate::infra::user_info::StoreAvatarRepository;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::infra::user_info::{FsAvatarRepository, FsAvatarRepositoryError};

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

#[cfg(feature = "test")]
pub mod util;
#[cfg(not(feature = "test"))]
pub(crate) mod util;
