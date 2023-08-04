use chrono::{NaiveDateTime, Utc};
use std::path::Path;
pub use std::path::PathBuf;

pub use jid::{BareJid, Error as JidParseError, FullJid};

pub use prose_core_client::types::{
    AccountSettings, Address, Availability, Emoji, MessageId, StanzaId, Url, UserActivity,
    UserProfile,
};
pub use prose_core_client::{CachePolicy, ConnectionEvent};
pub use prose_xmpp::ConnectionError;

pub use crate::types::{AccountBookmark, ClientEvent, Contact, DateTime, JID};
pub use crate::{
    account_bookmarks_client::AccountBookmarksClient, client::*, logger::*, ClientError,
};

impl UniffiCustomTypeConverter for MessageId {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(val.into())
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.into_inner()
    }
}

impl UniffiCustomTypeConverter for StanzaId {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(val.into())
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.into_inner()
    }
}

impl UniffiCustomTypeConverter for Emoji {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(val.into())
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.into_inner()
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

pub mod uniffi_types {
    pub use crate::{
        client::Client,
        types::{parse_jid, AccountBookmark, DateTime, Message, MessagesPage, Reaction, JID},
        AccountSettings, Availability, CachePolicy, ClientError, ConnectionError, Contact, Emoji,
        FullJid, JidParseError, MessageId, PathBuf, StanzaId, Url, UserProfile,
    };
}

uniffi::include_scaffolding!("prose_sdk_ffi");
