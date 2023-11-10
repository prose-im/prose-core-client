// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::services::RoomEnvelope;

#[derive(Clone, Debug, PartialEq)]
pub struct SidebarItem {
    pub name: String,
    pub room: RoomEnvelope,
    pub is_favorite: bool,
    pub has_draft: bool,
    pub unread_count: u32,
    pub error: Option<String>,
}
