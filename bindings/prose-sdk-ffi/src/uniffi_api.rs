// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use crate::types::{AccountBookmark, ClientError, ClientEvent, Contact, Group, Message, JID};
pub use crate::{account_bookmarks_client::AccountBookmarksClient, client::*, logger::*};
use jid::BareJid as CoreBareJid;
use mime::Mime as CoreMime;
use prose_core_client::dtos::{
    AvatarId as CoreAvatarId, Emoji as CoreEmoji, MessageId as CoreMessageId, MucId as CoreMucId,
    OccupantId as CoreOccupantId, ParticipantId as CoreParticipantId,
    PresenceSubRequestId as CorePresenceSubRequestId, RoomId as CoreRoomId,
    ServerId as CoreServerId, UnicodeScalarIndex as CoreUnicodeScalarIndex, Url as CoreUrl,
    UserId as CoreUserId,
};
use std::path::PathBuf as CorePathBuf;

pub struct PathBuf(String);
pub struct Url(String);
pub struct Emoji(String);
pub struct MessageId(String);
pub struct BareJid(String);
pub struct UserId(BareJid);
pub struct OccupantId(String);
pub struct MucId(BareJid);
pub struct DateTime(i64);
pub struct DateTimeFixed(i64);
pub struct Mime(String);
pub struct UnicodeScalarIndex(u64);
pub struct PresenceSubRequestId(UserId);
pub struct AvatarId(String);
pub struct ServerId(BareJid);

uniffi::custom_newtype!(PathBuf, String);
uniffi::custom_newtype!(Url, String);
uniffi::custom_newtype!(Emoji, String);
uniffi::custom_newtype!(MessageId, String);
uniffi::custom_newtype!(BareJid, String);
uniffi::custom_newtype!(UserId, BareJid);
uniffi::custom_newtype!(OccupantId, String);
uniffi::custom_newtype!(MucId, BareJid);
uniffi::custom_newtype!(DateTime, i64);
uniffi::custom_newtype!(DateTimeFixed, i64);
uniffi::custom_newtype!(Mime, String);
uniffi::custom_newtype!(UnicodeScalarIndex, u64);
uniffi::custom_newtype!(PresenceSubRequestId, UserId);
uniffi::custom_newtype!(AvatarId, String);
uniffi::custom_newtype!(ServerId, BareJid);

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

impl From<MessageId> for CoreMessageId {
    fn from(value: MessageId) -> Self {
        value.0.into()
    }
}

impl From<CoreMucId> for MucId {
    fn from(value: CoreMucId) -> Self {
        MucId(value.into_inner().into())
    }
}

impl From<CoreBareJid> for BareJid {
    fn from(value: CoreBareJid) -> Self {
        BareJid(value.to_string())
    }
}

impl From<BareJid> for CoreBareJid {
    fn from(value: BareJid) -> Self {
        value
            .0
            .as_str()
            .parse::<CoreBareJid>()
            .expect("BareJid is invalid")
    }
}

impl From<MucId> for CoreMucId {
    fn from(value: MucId) -> Self {
        CoreBareJid::from(value.0).into()
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
        UserId(value.into_inner().into())
    }
}

impl From<UserId> for CoreUserId {
    fn from(value: UserId) -> Self {
        CoreBareJid::from(value.0).into()
    }
}

impl From<CorePresenceSubRequestId> for PresenceSubRequestId {
    fn from(value: CorePresenceSubRequestId) -> Self {
        PresenceSubRequestId(value.to_user_id().into())
    }
}

impl From<PresenceSubRequestId> for CorePresenceSubRequestId {
    fn from(value: PresenceSubRequestId) -> Self {
        CoreUserId::from(value.0).into()
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

impl From<chrono::DateTime<chrono::FixedOffset>> for DateTimeFixed {
    fn from(value: chrono::DateTime<chrono::FixedOffset>) -> Self {
        DateTimeFixed(value.timestamp_millis())
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

impl From<Emoji> for CoreEmoji {
    fn from(value: Emoji) -> Self {
        value.0.into()
    }
}

impl From<CoreServerId> for ServerId {
    fn from(value: CoreServerId) -> Self {
        ServerId(value.into_inner().into())
    }
}

impl From<ServerId> for CoreServerId {
    fn from(value: ServerId) -> Self {
        CoreBareJid::from(value.0).into()
    }
}

impl From<CoreAvatarId> for AvatarId {
    fn from(value: CoreAvatarId) -> Self {
        AvatarId(value.to_string())
    }
}

impl From<AvatarId> for CoreAvatarId {
    fn from(value: AvatarId) -> Self {
        CoreAvatarId::from_str_unchecked(value.0)
    }
}

impl From<CorePathBuf> for PathBuf {
    fn from(value: CorePathBuf) -> Self {
        PathBuf(
            value
                .to_str()
                .expect("Could not convert path to str")
                .to_owned(),
        )
    }
}

pub mod uniffi_types {
    pub use crate::{
        client::Client,
        types::{parse_jid, AccountBookmark, Message, Reaction, UserProfile, JID},
        AvatarId, ClientError, Contact, DateTimeFixed, Emoji, MessageId, MucId, ParticipantId,
        PathBuf, PresenceSubRequestId, RoomId, ServerId, UnicodeScalarIndex, Url, UserId,
    };
}

uniffi::setup_scaffolding!();
