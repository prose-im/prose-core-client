// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::sidebar::models::{Bookmark, BookmarkType};
use crate::dtos::RoomId;

impl Bookmark {
    pub fn direct_message(jid: impl Into<RoomId>) -> Self {
        let jid = jid.into();

        Self {
            name: jid.to_display_name(),
            jid,
            r#type: BookmarkType::DirectMessage,
            is_favorite: false,
            in_sidebar: false,
        }
    }

    pub fn group(jid: impl Into<RoomId>, name: impl Into<String>) -> Self {
        let jid = jid.into();

        Self {
            name: name.into(),
            jid,
            r#type: BookmarkType::Group,
            is_favorite: false,
            in_sidebar: false,
        }
    }

    pub fn public_channel(jid: impl Into<RoomId>, name: impl Into<String>) -> Self {
        let jid = jid.into();

        Self {
            name: name.into(),
            jid,
            r#type: BookmarkType::PublicChannel,
            is_favorite: false,
            in_sidebar: false,
        }
    }

    pub fn private_channel(jid: impl Into<RoomId>, name: impl Into<String>) -> Self {
        let jid = jid.into();

        Self {
            name: name.into(),
            jid,
            r#type: BookmarkType::PrivateChannel,
            is_favorite: false,
            in_sidebar: false,
        }
    }
}

impl Bookmark {
    pub fn set_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn set_is_favorite(mut self, is_favorite: bool) -> Self {
        self.is_favorite = is_favorite;
        self
    }

    pub fn set_in_sidebar(mut self, in_sidebar: bool) -> Self {
        self.in_sidebar = in_sidebar;
        self
    }
}
