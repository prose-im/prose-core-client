// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::{Availability, Avatar, UserStatus};
use crate::FFIUserId;
use prose_core_client::dtos::{
    UserBasicInfo as CoreUserBasicInfo, UserPresenceInfo as CoreUserPresenceInfo,
};
use std::sync::Arc;

#[derive(uniffi::Record)]
pub struct UserPresenceInfo {
    pub id: FFIUserId,
    pub name: String,
    pub full_name: Option<String>,
    pub availability: Availability,
    pub avatar: Option<Arc<Avatar>>,
    pub status: Option<UserStatus>,
}

#[derive(uniffi::Record)]
pub struct UserBasicInfo {
    pub id: FFIUserId,
    pub name: String,
    pub avatar: Option<Arc<Avatar>>,
}

impl From<CoreUserBasicInfo> for UserBasicInfo {
    fn from(value: CoreUserBasicInfo) -> Self {
        UserBasicInfo {
            id: value.id.into(),
            name: value.name,
            avatar: value.avatar.map(|avatar| Arc::new(avatar.into())),
        }
    }
}

impl From<CoreUserPresenceInfo> for UserPresenceInfo {
    fn from(value: CoreUserPresenceInfo) -> Self {
        UserPresenceInfo {
            id: value.id.into(),
            name: value.name,
            full_name: value.full_name,
            availability: value.availability.into(),
            avatar: value.avatar.map(|avatar| Arc::new(avatar.into())),
            status: value.status.map(Into::into),
        }
    }
}
