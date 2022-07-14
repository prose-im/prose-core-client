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

pub use account::AccountObserver as XMPPAccountObserver;
pub use client::Client as XMPPClient;
pub use error::{Error as ProseError, Result, StanzaParseError};
pub use jid::{BareJid, JidParseError};
pub use libstrophe::Error as LibStropheError;
pub use logger::enable_logging;
pub use types::delay::Delay as XMPPDelay;
pub use types::forwarded_message::ForwardedMessage as XMPPForwardedMessage;
pub use types::mam::{
    DefaultBehavior as XMPPMAMDefaultBehavior, MAMPreferences as XMPPMAMPreferences,
};
pub use types::message::{
    ChatState as XMPPChatState, Message as XMPPMessage, MessageKind as XMPPMessageKind,
};
pub use types::presence::{
    Presence as XMPPPresence, PresenceKind as XMPPPresenceKind, ShowKind as XMPPShowKind,
};
pub use types::roster::{
    Roster as XMPPRoster, RosterGroup as XMPPRosterGroup, RosterItem as XMPPRosterItem,
    RosterItemSubscription as XMPPRosterItemSubscription,
};

pub use connection::{
    ConnectionEvent, ConnectionHandler, StanzaHandler, XMPPConnection, XMPPSender,
};

#[cfg(feature = "test-helpers")]
pub use account::{Account, AccountObserverMock};

pub fn parse_jid(jid_str: &str) -> Result<BareJid, JidParseError> {
    jid_str.parse::<BareJid>()
}

pub fn format_jid(jid: &BareJid) -> String {
    jid.to_string()
}

uniffi_macros::include_scaffolding!("ProseCoreClientFFI");
