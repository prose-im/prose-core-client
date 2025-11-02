// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::{Availability, Avatar, RoomState, UserStatus};
use crate::{HexColor, RoomId};
use prose_core_client::dtos::{
    SidebarItem as CoreSidebarItem, SidebarItemType as CoreSidebarItemType,
};
use std::sync::Arc;

#[derive(uniffi::Enum)]
pub enum SidebarItemType {
    DirectMessage {
        availability: Availability,
        initials: String,
        color: HexColor,
        avatar: Option<Arc<Avatar>>,
        status: Option<UserStatus>,
    },
    Group,
    PrivateChannel,
    PublicChannel,
    Generic,
}

#[derive(uniffi::Record)]
pub struct SidebarItem {
    pub name: String,
    pub room_id: RoomId,
    pub r#type: SidebarItemType,
    pub room_state: RoomState,
    pub is_favorite: bool,
    pub has_draft: bool,
    pub unread_count: u32,
    pub mentions_count: u32,
}

impl From<CoreSidebarItem> for SidebarItem {
    fn from(value: CoreSidebarItem) -> Self {
        let room = value.room.to_generic_room();

        SidebarItem {
            name: value.name,
            room_id: room.jid().clone().into(),
            r#type: value.r#type.into(),
            room_state: room.state().into(),
            is_favorite: value.is_favorite,
            has_draft: value.has_draft,
            unread_count: value.unread_count,
            mentions_count: value.mentions_count,
        }
    }
}

impl From<CoreSidebarItemType> for SidebarItemType {
    fn from(value: CoreSidebarItemType) -> Self {
        match value {
            CoreSidebarItemType::DirectMessage {
                availability,
                initials,
                color,
                avatar,
                status,
            } => SidebarItemType::DirectMessage {
                availability: availability.into(),
                initials,
                color: color.into(),
                avatar: avatar.map(|a| Arc::new(a.into())),
                status: status.map(Into::into),
            },
            CoreSidebarItemType::Group => SidebarItemType::Group,
            CoreSidebarItemType::PrivateChannel => SidebarItemType::PrivateChannel,
            CoreSidebarItemType::PublicChannel => SidebarItemType::PublicChannel,
            CoreSidebarItemType::Generic => SidebarItemType::Generic,
        }
    }
}
