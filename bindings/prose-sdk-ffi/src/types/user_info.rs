// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::Avatar;
use crate::UserId;
use prose_core_client::dtos::UserBasicInfo as CoreUserBasicInfo;
use std::sync::Arc;

#[derive(uniffi::Record)]
pub struct UserBasicInfo {
    pub id: UserId,
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
