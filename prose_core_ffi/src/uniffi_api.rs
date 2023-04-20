use std::path::Path;
pub use std::path::PathBuf;

use chrono::NaiveDateTime;
pub use jid::{BareJid, FullJid, JidParseError};

pub use prose_core_client::types::{
    Address, Availability, Contact, Emoji, Message, MessageId, Reaction, StanzaId, Url, UserProfile,
};
use prose_core_client::types::{DateTime as ChronoDateTime, Page, Utc};
pub use prose_core_client::{
    AccountBookmark, AccountBookmarksClient, CachePolicy, ClientDelegate, ClientEvent,
};
pub use prose_core_lib::{modules::roster::Subscription, ConnectionError, ConnectionEvent};

pub use crate::{client::*, logger::*, ClientError};

pub type DateTime = ChronoDateTime<Utc>;

#[derive(uniffi::Record)]
pub struct MessagesPage {
    pub messages: Vec<Message>,
    pub is_complete: bool,
}

impl From<Page<Message>> for MessagesPage {
    fn from(value: Page<Message>) -> Self {
        MessagesPage {
            messages: value.items,
            is_complete: value.is_complete,
        }
    }
}

impl UniffiCustomTypeConverter for PathBuf {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Path::new(&val).to_path_buf())
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_str()
            .expect("Could not convert path to str")
            .to_owned()
    }
}

impl UniffiCustomTypeConverter for Url {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Url::parse(&val).expect(&format!("Received invalid URL '{}'", val)))
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.into()
    }
}

impl UniffiCustomTypeConverter for DateTime {
    type Builtin = i64;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Self::from_utc(
            NaiveDateTime::from_timestamp_millis(val).expect("Received invalid timestamp"),
            Utc,
        ))
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.timestamp_millis()
    }
}

impl UniffiCustomTypeConverter for MessageId {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(val.into())
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.0
    }
}

impl UniffiCustomTypeConverter for StanzaId {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(val.into())
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.0
    }
}

impl UniffiCustomTypeConverter for Emoji {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(val.into())
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.0
    }
}

#[uniffi::export]
pub fn parse_jid(jid: String) -> Result<BareJid, JidParseError> {
    jid.parse::<BareJid>()
}

#[uniffi::export]
pub fn format_jid(jid: BareJid) -> String {
    jid.to_string()
}

pub mod uniffi_types {
    pub use crate::{
        client::Client, Availability, BareJid, CachePolicy, ClientError, ConnectionError, Contact,
        Emoji, FullJid, JidParseError, Message, MessageId, MessagesPage, PathBuf, StanzaId, Url,
        UserProfile,
    };
}

uniffi::include_scaffolding!("prose_core_ffi");
