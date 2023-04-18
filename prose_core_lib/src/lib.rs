pub use client::{Client, ConnectedClient};
pub use connector::Connection;
pub use errors::ConnectionError;
pub use handlers::{ConnectionEvent, ConnectionHandler, StanzaHandler, TimedHandler};

pub mod client;
mod connector;
mod dependencies;
mod errors;
mod handlers;
mod helpers;
pub mod modules;
pub mod stanza;
