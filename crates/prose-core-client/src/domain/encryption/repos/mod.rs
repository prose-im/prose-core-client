// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use encryption_keys_repository::EncryptionKeysRepository;
pub use user_device_repository::UserDeviceRepository;

pub mod encryption_keys_repository;
mod user_device_repository;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::encryption_keys_repository::MockEncryptionKeysRepository;
    pub use super::user_device_repository::MockUserDeviceRepository;
}
