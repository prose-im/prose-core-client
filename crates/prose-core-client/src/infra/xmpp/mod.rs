// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use xmpp_client::{XMPPClient, XMPPClientBuilder};

pub(crate) mod event_parser;
pub(crate) mod type_conversions;
mod xmpp_client;
