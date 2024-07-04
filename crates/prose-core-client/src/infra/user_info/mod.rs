// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use in_memory_user_info_repository::InMemoryUserInfoRepository;
pub(self) use presence_map::PresenceMap;
pub use user_profile_repository::{UserProfileRecord, UserProfileRepository};

#[cfg(not(target_arch = "wasm32"))]
pub use fs_avatar_repository::*;
#[cfg(target_arch = "wasm32")]
pub use store_avatar_repository::*;

mod in_memory_user_info_repository;
mod presence_map;
mod user_info_service;
mod user_profile_repository;

#[cfg(not(target_arch = "wasm32"))]
mod fs_avatar_repository;
#[cfg(target_arch = "wasm32")]
mod store_avatar_repository;

pub const MAX_IMAGE_DIMENSIONS: (u32, u32) = (400, 400);
