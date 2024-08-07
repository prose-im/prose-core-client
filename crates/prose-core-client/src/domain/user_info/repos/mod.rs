// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use avatar_repository::AvatarRepository;
pub use user_info_repository::{UpdateHandler, UserInfoRepository};
pub use user_profile_repository::UserProfileRepository;

mod avatar_repository;
mod user_info_repository;
pub mod user_profile_repository;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::avatar_repository::MockAvatarRepository;
    pub use super::user_info_repository::MockUserInfoRepository;
    pub use super::user_profile_repository::MockUserProfileRepository;
}
