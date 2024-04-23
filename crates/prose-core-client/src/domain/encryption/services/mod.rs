// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use encryption_domain_service::{EncryptionDomainService, EncryptionError};
pub use encryption_service::EncryptionService;
pub use user_device_id_provider::{RandUserDeviceIdProvider, UserDeviceIdProvider};
pub use user_device_service::UserDeviceService;

mod encryption_domain_service;
mod encryption_service;
pub mod impls;
mod user_device_id_provider;
mod user_device_service;

#[cfg(feature = "test")]
pub use user_device_id_provider::IncrementingUserDeviceIdProvider;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::encryption_domain_service::MockEncryptionDomainService;
    pub use super::encryption_service::MockEncryptionService;
    pub use super::user_device_service::MockUserDeviceService;
}
