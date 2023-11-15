// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::sidebar::models::{BookmarkType, SidebarItem};
use crate::dtos::RoomJid;
use crate::util::jid_ext::BareJidExt;

impl SidebarItem {
    pub fn direct_message(jid: impl Into<RoomJid>) -> Self {
        let jid = jid.into();

        Self {
            name: jid.to_display_name(),
            jid,
            r#type: BookmarkType::DirectMessage,
            is_favorite: false,
            error: None,
        }
    }

    pub fn group(jid: impl Into<RoomJid>, name: impl Into<String>) -> Self {
        let jid = jid.into();

        Self {
            name: name.into(),
            jid,
            r#type: BookmarkType::Group,
            is_favorite: false,
            error: None,
        }
    }

    pub fn public_channel(jid: impl Into<RoomJid>, name: impl Into<String>) -> Self {
        let jid = jid.into();

        Self {
            name: name.into(),
            jid,
            r#type: BookmarkType::PublicChannel,
            is_favorite: false,
            error: None,
        }
    }

    pub fn private_channel(jid: impl Into<RoomJid>, name: impl Into<String>) -> Self {
        let jid = jid.into();

        Self {
            name: name.into(),
            jid,
            r#type: BookmarkType::PrivateChannel,
            is_favorite: false,
            error: None,
        }
    }
}

impl SidebarItem {
    pub fn set_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn set_is_favorite(mut self, is_favorite: bool) -> Self {
        self.is_favorite = is_favorite;
        self
    }
}
