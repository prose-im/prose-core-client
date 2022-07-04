// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)
mod account;
mod client;
mod connection;
mod error;
mod extensions;
mod helpers;
mod logger;
mod types;

#[cfg(feature = "test-helpers")]
pub mod test_helpers;

pub use account::{Account, AccountObserver};
pub use client::Client;
pub use error::{Error as ProseError, Result, StanzaParseError};
pub use jid::{BareJid, JidParseError};
pub use libstrophe::Error as LibStropheError;
pub use logger::enable_logging;
pub use types::message::{ChatState, Message, MessageKind};
pub use types::presence::{Presence, PresenceKind, ShowKind};
pub use types::roster::{Roster, RosterGroup, RosterItem, RosterItemSubscription};

pub use connection::{
    ConnectionEvent, ConnectionHandler, StanzaHandler, XMPPConnection, XMPPSender,
};

pub fn parse_jid(jid_str: &str) -> Result<BareJid, JidParseError> {
    jid_str.parse::<BareJid>()
}

pub fn format_jid(jid: &BareJid) -> String {
    jid.to_string()
}

uniffi_macros::include_scaffolding!("ProseCoreClientFFI");
