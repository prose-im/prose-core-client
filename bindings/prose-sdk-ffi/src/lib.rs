// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_core_client::FsAvatarCacheError;
pub use uniffi_api::*;

mod account_bookmarks_client;
mod client;
mod logger;
mod types;
mod uniffi_api;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("client error: {msg}")]
    Generic { msg: String },
}

impl From<anyhow::Error> for ClientError {
    fn from(e: anyhow::Error) -> ClientError {
        ClientError::Generic { msg: e.to_string() }
    }
}

impl From<FsAvatarCacheError> for ClientError {
    fn from(e: FsAvatarCacheError) -> Self {
        ClientError::Generic { msg: e.to_string() }
    }
}
