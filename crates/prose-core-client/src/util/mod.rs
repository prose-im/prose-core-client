// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use form_config::FormConfig;
pub use string_ext::StringExt;

#[cfg(not(target_arch = "wasm32"))]
pub mod account_bookmarks_client;

pub mod form_config;
pub mod jid_ext;
pub mod string_ext;
