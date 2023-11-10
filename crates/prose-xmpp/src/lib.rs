// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use client::{Client, ClientBuilder};
pub use connector::{Connection, ConnectionError, Connector};
pub use deps::{IDProvider, SystemTimeProvider, TimeProvider, UUIDProvider};
pub use event::Event;
pub use stanza::ns;
pub use util::{parse_bool, ElementExt, ParseError, PublishOptionsExt, RequestError};

pub mod client;
pub mod connector;
mod deps;
mod event;
pub mod mods;
pub mod stanza;
mod util;

#[cfg(feature = "test")]
pub mod test;
