pub use avatar_cache::{AvatarCache, IMAGE_OUTPUT_MIME_TYPE, MAX_IMAGE_DIMENSIONS};
pub use data_cache::{ContactsCache, DataCache, MessageCache};
pub use noop_avatar_cache::NoopAvatarCache;
pub use noop_data_cache::NoopDataCache;

mod avatar_cache;
mod data_cache;
mod noop_avatar_cache;
mod noop_data_cache;

#[cfg(feature = "native-app")]
pub mod fs_avatar_cache;

#[cfg(any(feature = "native-app", feature = "test-helpers"))]
pub mod sqlite_data_cache;
