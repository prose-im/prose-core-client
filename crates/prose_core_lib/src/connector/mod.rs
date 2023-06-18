pub use connector::{
    Connection, ConnectionError, ConnectionEvent, ConnectionEventHandler, Connector,
};

mod connector;

#[cfg(target_arch = "wasm32")]
pub mod strophe_js;
#[cfg(not(target_arch = "wasm32"))]
pub mod xmpp_rs;
