// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod account;
mod account_observer;
mod client;
mod logger;
mod types;

pub use account::ConnectionError;
pub use account_observer::AccountObserver;
pub use client::Client;
pub use jid::{BareJid, JidParseError};
pub use logger::enableLogging;
pub use types::message::{ChatState, Message, MessageKind};
pub use types::presence::{Presence, PresenceKind, ShowKind};
pub use types::roster::{Roster, RosterGroup, RosterItem, RosterItemSubscription};

#[allow(non_snake_case)]
pub fn parseJID(jid_str: &str) -> Result<BareJid, JidParseError> {
    jid_str.parse::<BareJid>()
}

#[allow(non_snake_case)]
pub fn formatJID(jid: &BareJid) -> String {
    jid.to_string()
}

uniffi_macros::include_scaffolding!("ProseCoreClientFFI");
