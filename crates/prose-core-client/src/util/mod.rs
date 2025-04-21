// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use coalesce_client_events::coalesce_client_events;
pub use form_config::FormConfig;
pub use join_all::join_all;
pub use path_ext::PathExt;
#[cfg(feature = "debug")]
pub use proxy_transformer::RandomDelayProxyTransformer;
pub use string_ext::StringExt;

#[cfg(not(target_arch = "wasm32"))]
pub mod account_bookmarks_client;

mod coalesce_client_events;
pub mod form_config;
pub mod jid_workspace;
mod join_all;
pub mod mime_serde_shim;
mod path_ext;
mod proxy_transformer;
pub mod string_ext;
