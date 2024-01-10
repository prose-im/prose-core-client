// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::rooms::models::RoomSidebarState;
use crate::domain::shared::models::RoomId;

use super::BookmarkType;

#[derive(Debug, Clone, PartialEq)]
pub struct Bookmark {
    pub name: String,
    pub jid: RoomId,
    pub r#type: BookmarkType,
    pub sidebar_state: RoomSidebarState,
}
