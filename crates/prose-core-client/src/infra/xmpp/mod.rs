// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use xmpp_client::{XMPPClient, XMPPClientBuilder};

pub mod event_parser;
pub mod type_conversions;
pub mod util;
pub mod xmpp_client;
