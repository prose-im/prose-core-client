pub use client::{Client, ClientBuilder};
pub use connector::{Connection, ConnectionError, Connector};
pub use deps::{IDProvider, SystemTimeProvider, TimeProvider, UUIDProvider};
pub use event::Event;
pub use stanza::ns;

pub mod client;
pub mod connector;
mod deps;
mod event;
pub mod mods;
pub mod stanza;
mod util;

#[cfg(feature = "test-helpers")]
pub mod test;
