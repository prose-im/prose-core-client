// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::rooms::models::RoomSidebarState;
use crate::domain::sidebar::models::{Bookmark, BookmarkType};
use crate::dtos::RoomId;

impl Bookmark {
    pub fn direct_message(jid: impl Into<RoomId>) -> Self {
        let jid = jid.into();

        Self {
            name: jid.to_display_name(),
            jid,
            r#type: BookmarkType::DirectMessage,
            sidebar_state: RoomSidebarState::NotInSidebar,
        }
    }

    pub fn group(jid: impl Into<RoomId>, name: impl Into<String>) -> Self {
        let jid = jid.into();

        Self {
            name: name.into(),
            jid,
            r#type: BookmarkType::Group,
            sidebar_state: RoomSidebarState::NotInSidebar,
        }
    }

    pub fn public_channel(jid: impl Into<RoomId>, name: impl Into<String>) -> Self {
        let jid = jid.into();

        Self {
            name: name.into(),
            jid,
            r#type: BookmarkType::PublicChannel,
            sidebar_state: RoomSidebarState::NotInSidebar,
        }
    }

    pub fn private_channel(jid: impl Into<RoomId>, name: impl Into<String>) -> Self {
        let jid = jid.into();

        Self {
            name: name.into(),
            jid,
            r#type: BookmarkType::PrivateChannel,
            sidebar_state: RoomSidebarState::NotInSidebar,
        }
    }
}

impl Bookmark {
    pub fn set_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn set_sidebar_state(mut self, state: RoomSidebarState) -> Self {
        self.sidebar_state = state;
        self
    }
}
