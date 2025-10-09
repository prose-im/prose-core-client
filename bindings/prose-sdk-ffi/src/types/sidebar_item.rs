// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::RoomEnvelope;
use prose_core_client::dtos::SidebarItem as CoreSidebarItem;

#[derive(uniffi::Record)]
pub struct SidebarItem {
    pub name: String,
    pub room: RoomEnvelope,
    pub is_favorite: bool,
    pub has_draft: bool,
    pub unread_count: u32,
    pub mentions_count: u32,
}

impl From<CoreSidebarItem> for SidebarItem {
    fn from(value: CoreSidebarItem) -> Self {
        SidebarItem {
            name: value.name,
            room: value.room.into(),
            is_favorite: value.is_favorite,
            has_draft: value.has_draft,
            unread_count: value.unread_count,
            mentions_count: value.mentions_count,
        }
    }
}
