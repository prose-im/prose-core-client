// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use client::Client;
pub(super) use client::ClientInner;
pub use client_builder::{ClientBuilder, UndefinedAvatarCache, UndefinedDataCache};
pub(super) use client_event::ReceivedMessage;

mod client;
mod client_builder;
mod client_contacts;
mod client_conversation;
mod client_event;
mod client_muc;
mod client_profile;
mod client_status;
