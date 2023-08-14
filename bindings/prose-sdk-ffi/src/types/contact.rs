// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::JID;
use prose_core_client::types::{
    roster::Group as CoreGroup, Availability, Contact as CoreContact, UserActivity,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Group {
    Favorite,
    Team,
    Other,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Contact {
    pub jid: JID,
    pub name: String,
    pub availability: Availability,
    pub activity: Option<UserActivity>,
    pub group: Group,
}

impl From<CoreContact> for Contact {
    fn from(value: CoreContact) -> Self {
        Contact {
            jid: value.jid.into(),
            name: value.name,
            availability: value.availability,
            activity: value.activity,
            group: value.group.into(),
        }
    }
}

impl From<CoreGroup> for Group {
    fn from(value: CoreGroup) -> Self {
        match value {
            CoreGroup::Favorite => Group::Favorite,
            CoreGroup::Team => Group::Team,
            CoreGroup::Other => Group::Other,
        }
    }
}
