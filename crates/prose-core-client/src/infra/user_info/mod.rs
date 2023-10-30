// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use caching_user_info_repository::{CachingUserInfoRepository, UserInfoRecord};
pub(self) use presence_map::PresenceMap;

pub mod caching_avatar_repository;
mod caching_user_info_repository;
mod presence_map;
mod user_info_service;
