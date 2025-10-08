// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use crate::types::{AccountBookmark, ClientError, ClientEvent, Contact, Group, Message, JID};
pub use crate::{account_bookmarks_client::AccountBookmarksClient, client::*, logger::*};
use mime::Mime as CoreMime;
use prose_core_client::dtos::{
    Emoji as CoreEmoji, MessageId as CoreMessageId, MucId as CoreMucId,
    OccupantId as CoreOccupantId, ParticipantId as CoreParticipantId, RoomId as CoreRoomId,
    UnicodeScalarIndex as CoreUnicodeScalarIndex, Url as CoreUrl, UserId as CoreUserId,
};

pub struct PathBuf(String);
pub struct Url(String);
pub struct Emoji(String);
pub struct MessageId(String);
pub struct UserId(String);
pub struct OccupantId(String);
pub struct MucId(String);
pub struct DateTime(i64);
pub struct Mime(String);
pub struct UnicodeScalarIndex(u64);

uniffi::custom_newtype!(PathBuf, String);
uniffi::custom_newtype!(Url, String);
uniffi::custom_newtype!(Emoji, String);
uniffi::custom_newtype!(MessageId, String);
uniffi::custom_newtype!(UserId, String);
uniffi::custom_newtype!(OccupantId, String);
uniffi::custom_newtype!(MucId, String);
uniffi::custom_newtype!(DateTime, i64);
uniffi::custom_newtype!(Mime, String);
uniffi::custom_newtype!(UnicodeScalarIndex, u64);

#[derive(uniffi::Enum)]
pub enum RoomId {
    User(UserId),
    Muc(MucId),
}

#[derive(uniffi::Enum)]
pub enum ParticipantId {
    User(UserId),
    Occupant(OccupantId),
}

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

impl From<CoreMucId> for MucId {
    fn from(value: CoreMucId) -> Self {
        MucId(value.to_string())
    }
}

impl From<CoreRoomId> for RoomId {
    fn from(value: CoreRoomId) -> Self {
        match value {
            CoreRoomId::User(id) => RoomId::User(id.into()),
            CoreRoomId::Muc(id) => RoomId::Muc(id.into()),
        }
    }
}

impl From<CoreParticipantId> for ParticipantId {
    fn from(value: CoreParticipantId) -> Self {
        match value {
            CoreParticipantId::User(id) => ParticipantId::User(id.into()),
            CoreParticipantId::Occupant(id) => ParticipantId::Occupant(id.into()),
        }
    }
}

impl From<CoreUserId> for UserId {
    fn from(value: CoreUserId) -> Self {
        UserId(value.to_string())
    }
}

impl From<CoreOccupantId> for OccupantId {
    fn from(value: CoreOccupantId) -> Self {
        OccupantId(value.to_string())
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

impl From<CoreMime> for Mime {
    fn from(value: CoreMime) -> Self {
        Mime(value.to_string())
    }
}

impl From<Mime> for CoreMime {
    fn from(value: Mime) -> Self {
        value.0.parse().unwrap_or(mime::APPLICATION_OCTET_STREAM)
    }
}

impl From<CoreUnicodeScalarIndex> for UnicodeScalarIndex {
    fn from(value: CoreUnicodeScalarIndex) -> Self {
        UnicodeScalarIndex(*value.as_ref() as u64)
    }
}

impl From<CoreEmoji> for Emoji {
    fn from(value: CoreEmoji) -> Self {
        Emoji(value.into_inner())
    }
}

pub mod uniffi_types {
    pub use crate::{
        client::Client,
        types::{parse_jid, AccountBookmark, Message, Reaction, UserProfile, JID},
        ClientError, Contact, Emoji, MessageId, MucId, ParticipantId, PathBuf, RoomId,
        UnicodeScalarIndex, Url, UserId,
    };
}

uniffi::setup_scaffolding!();
