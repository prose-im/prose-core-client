// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use avatar_cache::{AvatarCache, MAX_IMAGE_DIMENSIONS};
#[cfg(not(target_arch = "wasm32"))]
pub use fs_avatar_cache::*;

mod avatar_cache;
#[cfg(not(target_arch = "wasm32"))]
mod fs_avatar_cache;
#[cfg(target_arch = "wasm32")]
mod store_avatar_cache;
