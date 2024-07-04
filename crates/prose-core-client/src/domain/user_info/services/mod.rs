// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use user_info_domain_service::UserInfoDomainService;
pub use user_info_service::UserInfoService;

pub mod impls;
mod user_info_domain_service;
mod user_info_service;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::user_info_domain_service::MockUserInfoDomainService;
    pub use super::user_info_service::MockUserInfoService;
}
