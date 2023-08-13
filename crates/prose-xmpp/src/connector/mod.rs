// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use connector::{
    Connection, ConnectionError, ConnectionEvent, ConnectionEventHandler, Connector,
};

mod connector;

#[cfg(not(target_arch = "wasm32"))]
pub mod xmpp_rs;
