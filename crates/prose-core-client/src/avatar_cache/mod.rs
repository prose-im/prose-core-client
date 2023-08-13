// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use avatar_cache::{AvatarCache, MAX_IMAGE_DIMENSIONS};
pub use noop_avatar_cache::NoopAvatarCache;

mod avatar_cache;
mod noop_avatar_cache;

#[cfg(not(target_arch = "wasm32"))]
pub mod fs_avatar_cache;
