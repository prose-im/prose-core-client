// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Formatter};

use crate::dtos::{Availability, AvatarBundle, RoomEnvelope, UserId, UserStatus};

#[derive(Clone, PartialEq)]
pub enum SidebarItemType {
    DirectMessage {
        user_id: UserId,
        availability: Availability,
        avatar_bundle: AvatarBundle,
        status: Option<UserStatus>,
    },
    Group,
    PrivateChannel,
    PublicChannel,
    Generic,
}

#[derive(Clone, PartialEq)]
pub struct SidebarItem {
    pub name: String,
    pub room: RoomEnvelope,
    pub r#type: SidebarItemType,
    pub is_favorite: bool,
    pub has_draft: bool,
    pub unread_count: u32,
    pub mentions_count: u32,
}

impl Debug for SidebarItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("id", &self.room.to_generic_room().data.room_id)
            .field("name", &self.name)
            .field("type", &self.room.to_generic_room().data.r#type)
            .field("is_favorite", &self.is_favorite)
            .field("has_draft", &self.has_draft)
            .field("unread_count", &self.unread_count)
            .field("mentions_count", &self.mentions_count)
            .finish()
    }
}
