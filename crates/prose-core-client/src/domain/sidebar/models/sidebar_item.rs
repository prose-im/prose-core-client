// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::RoomJid;

use super::BookmarkType;

#[derive(Debug, Clone, PartialEq)]
pub struct SidebarItem {
    pub name: String,
    pub jid: RoomJid,
    pub r#type: BookmarkType,
    pub is_favorite: bool,
    /// If we were unable to connect to a Room, `error` contains an error message about what
    /// went wrong.
    pub error: Option<String>,
}
