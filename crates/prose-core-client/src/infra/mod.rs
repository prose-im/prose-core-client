// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod account;
pub mod connection;
pub mod contacts;
pub mod encryption;
pub mod events;
pub mod general;
pub mod messaging;
pub mod platform_dependencies;
pub mod rooms;
pub mod settings;
pub mod sidebar;
pub mod uploads;
pub mod user_info;
pub mod workspace;
pub mod xmpp;

pub(crate) mod constants {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) use super::user_info::MAX_IMAGE_DIMENSIONS;
}
