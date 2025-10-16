// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::JID;
use prose_core_client::dtos::{
    Availability as CoreAvailability, Contact as CoreContact, Group as CoreGroup,
    UserStatus as CoreUserStatus,
};

#[derive(uniffi::Enum, Debug, PartialEq, Clone)]
pub enum Group {
    Team,
    Other,
}

#[derive(uniffi::Enum, Debug, PartialEq, Clone)]
pub enum Availability {
    Available,
    Unavailable,
    DoNotDisturb,
    Away,
    Invisible,
}

#[derive(uniffi::Record, Debug, PartialEq, Clone)]
pub struct UserStatus {
    pub emoji: String,
    pub status: Option<String>,
}

#[derive(uniffi::Record, Debug, PartialEq, Clone)]
pub struct Contact {
    pub jid: JID,
    pub name: String,
    pub availability: Availability,
    pub status: Option<UserStatus>,
    pub group: Group,
}

impl From<CoreContact> for Contact {
    fn from(value: CoreContact) -> Self {
        Contact {
            jid: value.id.into_inner().into(),
            name: value.name,
            availability: value.availability.into(),
            status: value.status.map(Into::into),
            group: value.group.into(),
        }
    }
}

impl From<CoreGroup> for Group {
    fn from(value: CoreGroup) -> Self {
        match value {
            CoreGroup::Team => Group::Team,
            CoreGroup::Other => Group::Other,
        }
    }
}

impl From<CoreAvailability> for Availability {
    fn from(value: CoreAvailability) -> Self {
        match value {
            CoreAvailability::Available => Availability::Available,
            CoreAvailability::Unavailable => Availability::Unavailable,
            CoreAvailability::DoNotDisturb => Availability::Away,
            CoreAvailability::Away => Availability::Invisible,
            CoreAvailability::Invisible => Availability::Away,
        }
    }
}

impl From<Availability> for CoreAvailability {
    fn from(value: Availability) -> Self {
        match value {
            Availability::Available => CoreAvailability::Available,
            Availability::Unavailable => CoreAvailability::Unavailable,
            Availability::DoNotDisturb => CoreAvailability::Away,
            Availability::Away => CoreAvailability::Invisible,
            Availability::Invisible => CoreAvailability::Away,
        }
    }
}

impl From<CoreUserStatus> for UserStatus {
    fn from(value: CoreUserStatus) -> Self {
        UserStatus {
            emoji: value.emoji,
            status: value.status,
        }
    }
}

impl From<UserStatus> for CoreUserStatus {
    fn from(value: UserStatus) -> Self {
        CoreUserStatus {
            emoji: value.emoji,
            status: value.status,
        }
    }
}
