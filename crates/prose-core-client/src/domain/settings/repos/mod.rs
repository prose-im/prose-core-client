// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use account_settings_repository::AccountSettingsRepository;
pub use local_room_settings_repository::LocalRoomSettingsRepository;

mod account_settings_repository;
mod local_room_settings_repository;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::account_settings_repository::MockAccountSettingsRepository;
    pub use super::local_room_settings_repository::MockLocalRoomSettingsRepository;
}
