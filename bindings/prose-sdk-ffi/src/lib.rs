use prose_core_client::avatar_cache::fs_avatar_cache::FsAvatarCacheError;
use prose_core_client::data_cache::sqlite::SQLiteCacheError;
pub use uniffi_api::*;

mod client;
mod logger;
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

impl From<SQLiteCacheError> for ClientError {
    fn from(e: SQLiteCacheError) -> Self {
        ClientError::Generic { msg: e.to_string() }
    }
}

impl From<FsAvatarCacheError> for ClientError {
    fn from(e: FsAvatarCacheError) -> Self {
        ClientError::Generic { msg: e.to_string() }
    }
}
