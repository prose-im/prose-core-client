// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use caching_user_device_repository::{CachingUserDeviceRepository, UserDeviceRecord};
pub use encryption_key_records::{
    KyberPreKeyRecord, LocalDeviceRecord, PreKeyRecord, SenderKeyRecord, SessionRecord,
    SignedPreKeyRecord,
};
pub use encryption_keys_repository::EncryptionKeysRepository;
pub use session_repository::SessionRepository;

mod caching_user_device_repository;
mod encryption_key_records;
mod encryption_keys_repository;
mod session_repository;
mod user_device_service;
