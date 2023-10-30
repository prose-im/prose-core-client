// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use account_bookmark::AccountBookmark;
pub use client_event::ClientEvent;
pub use contact::{Contact, Group};
pub use jid::{parse_jid, JID};
pub use message::{DateTime, Message, Reaction};

mod account_bookmark;
mod client_event;
mod contact;
mod jid;
mod message;
