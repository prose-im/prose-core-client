// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use user_profile_service::UserProfileService;

mod user_profile_service;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::user_profile_service::MockUserProfileService;
}
