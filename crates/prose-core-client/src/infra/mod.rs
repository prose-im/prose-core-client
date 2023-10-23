// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub mod account;
pub mod avatars;
pub mod contacts;
pub mod general;
pub mod messaging;
pub mod platform_dependencies;
pub mod rooms;
pub mod settings;
pub mod user_info;
pub mod user_profile;
pub mod xmpp;

pub(crate) mod constants {
    pub(crate) use super::avatars::{
        IMAGE_OUTPUT_FORMAT, IMAGE_OUTPUT_MIME_TYPE, MAX_IMAGE_DIMENSIONS,
    };
}
