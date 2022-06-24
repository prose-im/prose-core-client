// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod account;
mod account_observer;
mod client;
mod logger;
mod types;

pub use account_observer::AccountObserver;
pub use client::Client;
pub use jid::BareJid;
pub use logger::enableLogging;
pub use types::message::{ChatState, Message, MessageKind};
pub use types::presence::{Presence, PresenceKind, ShowKind};
pub use types::roster::{Roster, RosterGroup, RosterItem, RosterItemSubscription};

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("JID is invalid")]
    InvalidJID,
}

uniffi_macros::include_scaffolding!("ProseCoreClientFFI");
