use std::path::PathBuf;

pub use chrono::{DateTime, Utc};
#[cfg(not(feature = "typescript"))]
use jid::BareJid;
use microtype::microtype;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use typeshare::typeshare;
pub use url::Url;

#[cfg(feature = "typescript")]
use crate::jid::BareJid;

mod ext;
#[cfg(feature = "typescript")]
mod jid;

#[typeshare]
#[derive(Debug, Display, PartialEq, Serialize, Deserialize, Clone)]
pub enum Availability {
    Available,
    Unavailable,
    DoNotDisturb,
    Away,
}

#[typeshare]
#[derive(Debug, PartialEq, Clone)]
pub struct Contact {
    pub jid: BareJid,
    pub name: String,
    pub avatar: Option<PathBuf>,
    pub availability: Availability,
    pub status: Option<String>,
    pub groups: Vec<String>,
}

#[typeshare]
#[derive(Debug, PartialEq, Clone)]
pub struct Address {
    pub locality: Option<String>,
    pub country: Option<String>,
}

#[typeshare]
#[derive(Debug, PartialEq, Clone)]
pub struct UserProfile {
    pub full_name: Option<String>,
    pub nickname: Option<String>,
    pub org: Option<String>,
    pub title: Option<String>,
    pub email: Option<String>,
    pub tel: Option<String>,
    pub url: Option<Url>,
    pub address: Option<Address>,
}

microtype! {
    pub String {
        #[string]
        #[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize)]
        MessageId,

        #[string]
        #[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize)]
        StanzaId,

        #[string]
        #[derive(Debug, PartialEq, Clone, Serialize)]
        Emoji
    }
}

#[cfg(feature = "typescript")]
#[typeshare]
type MessageId = String;

#[cfg(feature = "typescript")]
#[typeshare]
type Reaction = String;

#[typeshare]
#[derive(Debug, PartialEq, Serialize)]
pub struct Reaction {
    pub emoji: Emoji,
    pub from: Vec<BareJid>,
}

#[typeshare]
#[derive(Debug, PartialEq, Serialize)]
pub struct Message {
    pub id: MessageId,
    pub stanza_id: Option<StanzaId>,
    pub from: BareJid,
    pub body: String,
    pub timestamp: DateTime<Utc>,
    pub is_read: bool,
    pub is_edited: bool,
    pub is_delivered: bool,
    pub reactions: Vec<Reaction>,
}
