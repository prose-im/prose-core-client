pub use connector::{
    Connection, ConnectionError, ConnectionEvent, ConnectionEventHandler, Connector,
};

mod connector;

#[cfg(not(target_arch = "wasm32"))]
pub mod xmpp_rs;
