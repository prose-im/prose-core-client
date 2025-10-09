// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::MucId;
use prose_core_client::dtos::PublicRoomInfo as CorePublicRoomInfo;

#[derive(uniffi::Record)]
pub struct PublicRoomInfo {
    pub id: MucId,
    pub name: Option<String>,
}

impl From<CorePublicRoomInfo> for PublicRoomInfo {
    fn from(value: CorePublicRoomInfo) -> Self {
        PublicRoomInfo {
            id: value.id.into(),
            name: value.name,
        }
    }
}
