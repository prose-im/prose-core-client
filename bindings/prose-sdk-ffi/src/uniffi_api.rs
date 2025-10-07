// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use jid::{BareJid, Error as JidParseError, FullJid};

pub use prose_core_client::dtos::{MessageId as CoreMessageId, Url as CoreUrl};
pub use prose_core_client::ConnectionEvent;

pub use crate::types::{AccountBookmark, ClientError, ClientEvent, Contact, Group, Message, JID};
pub use crate::{account_bookmarks_client::AccountBookmarksClient, client::*, logger::*};

pub struct PathBuf(String);
pub struct Url(String);
pub struct Emoji(String);
pub struct MessageId(String);
pub struct DateTime(i64);

uniffi::custom_type!(PathBuf, String);
uniffi::custom_type!(Url, String);
uniffi::custom_type!(Emoji, String);
uniffi::custom_type!(MessageId, String);
uniffi::custom_type!(DateTime, i64);

impl PathBuf {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<CoreMessageId> for MessageId {
    fn from(value: CoreMessageId) -> Self {
        MessageId(value.into_inner())
    }
}

impl From<chrono::DateTime<chrono::Utc>> for DateTime {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        DateTime(value.timestamp_millis())
    }
}

impl From<CoreUrl> for Url {
    fn from(value: CoreUrl) -> Self {
        Url(value.into())
    }
}

impl From<Url> for CoreUrl {
    fn from(value: Url) -> Self {
        CoreUrl::parse(value.0.as_str())
            .expect(&format!("Received invalid URL '{}'", value.0.as_str()))
    }
}

impl From<String> for PathBuf {
    fn from(value: String) -> Self {
        PathBuf(value)
    }
}

impl From<PathBuf> for String {
    fn from(value: PathBuf) -> Self {
        value.0
    }
}

impl From<String> for Url {
    fn from(value: String) -> Self {
        Url(value)
    }
}

impl From<Url> for String {
    fn from(value: Url) -> Self {
        value.0
    }
}

impl From<String> for Emoji {
    fn from(value: String) -> Self {
        Emoji(value)
    }
}

impl From<Emoji> for String {
    fn from(value: Emoji) -> Self {
        value.0
    }
}

impl From<String> for MessageId {
    fn from(value: String) -> Self {
        MessageId(value)
    }
}

impl From<MessageId> for String {
    fn from(value: MessageId) -> Self {
        value.0
    }
}

impl From<i64> for DateTime {
    fn from(value: i64) -> Self {
        DateTime(value)
    }
}

impl From<DateTime> for i64 {
    fn from(value: DateTime) -> Self {
        value.0
    }
}

pub mod uniffi_types {
    pub use crate::{
        client::Client,
        types::{parse_jid, AccountBookmark, Message, Reaction, UserProfile, JID},
        ClientError, Contact, Emoji, FullJid, JidParseError, MessageId, PathBuf, Url,
    };
}

uniffi::setup_scaffolding!();
