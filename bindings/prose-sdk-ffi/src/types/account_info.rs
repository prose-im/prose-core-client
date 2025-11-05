// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::{Availability, Avatar, UserStatus};
use crate::FFIUserId;
use prose_core_client::dtos::AccountInfo as CoreAccountInfo;
use std::sync::Arc;

#[derive(uniffi::Record)]
pub struct AccountInfo {
    pub id: FFIUserId,
    pub name: String,
    pub avatar: Option<Arc<Avatar>>,
    pub availability: Availability,
    pub status: Option<UserStatus>,
}

impl From<CoreAccountInfo> for AccountInfo {
    fn from(value: CoreAccountInfo) -> Self {
        AccountInfo {
            id: value.id.into(),
            name: value.name,
            avatar: value.avatar.map(|avatar| Arc::new(avatar.into())),
            availability: value.availability.into(),
            status: value.status.map(Into::into),
        }
    }
}
