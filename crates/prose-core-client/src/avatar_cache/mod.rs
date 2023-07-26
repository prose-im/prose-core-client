pub use avatar_cache::{AvatarCache, MAX_IMAGE_DIMENSIONS};
pub use noop_avatar_cache::NoopAvatarCache;

mod avatar_cache;
mod noop_avatar_cache;

#[cfg(feature = "native-app")]
pub mod fs_avatar_cache;
