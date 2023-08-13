// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use client::Client;
pub use client_builder::{ClientBuilder, UndefinedAvatarCache, UndefinedDataCache};

mod client;
mod client_builder;
mod client_contacts;
mod client_conversation;
mod client_event;
mod client_profile;
mod client_status;
