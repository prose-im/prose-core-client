// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use connector::{
    Connection, ConnectionError, ConnectionEvent, ConnectionEventHandler, Connector,
};
pub use proxy_connector::{ProxyConnector, ProxyTransformer};

mod connector;

mod proxy_connector;
#[cfg(not(target_arch = "wasm32"))]
pub mod xmpp_rs;
