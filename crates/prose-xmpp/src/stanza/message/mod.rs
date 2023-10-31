// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use xmpp_parsers::message::MessageType;

pub use fallback::Fallback;
pub use forwarding::Forwarded;
pub use message::{Id, Message};
pub use reactions::{Emoji, Reactions};

mod builder;
pub mod carbons;
pub mod chat_marker;
mod fallback;
pub mod fasten;
mod forwarding;
pub mod mam;
mod message;
mod reactions;
pub mod retract;
pub mod stanza_id;
