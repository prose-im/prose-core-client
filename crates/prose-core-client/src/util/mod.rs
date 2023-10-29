// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub(crate) use form_config::FormConfig;
pub(crate) use string_ext::StringExt;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod account_bookmarks_client;

pub(crate) mod form_config;
pub(crate) mod jid_ext;
pub(crate) mod string_ext;
