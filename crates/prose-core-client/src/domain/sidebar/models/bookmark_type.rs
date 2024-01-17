// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

use crate::domain::shared::models::RoomType;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum BookmarkType {
    DirectMessage,
    Group,
    PrivateChannel,
    PublicChannel,
    Generic,
}

impl Display for BookmarkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BookmarkType::DirectMessage => "Direct Message",
                BookmarkType::Group => "Group",
                BookmarkType::PrivateChannel => "Private Channel",
                BookmarkType::PublicChannel => "Public Channel",
                BookmarkType::Generic => "Generic",
            }
        )
    }
}

impl From<RoomType> for BookmarkType {
    fn from(value: RoomType) -> Self {
        match value {
            RoomType::Unknown => {
                unreachable!("Cannot build a bookmark from an unknown room type.")
            }
            RoomType::DirectMessage => BookmarkType::DirectMessage,
            RoomType::Group => BookmarkType::Group,
            RoomType::PrivateChannel => BookmarkType::PrivateChannel,
            RoomType::PublicChannel => BookmarkType::PublicChannel,
            RoomType::Generic => BookmarkType::Generic,
        }
    }
}
