// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

use crate::domain::sidebar::models::BookmarkType;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum RoomType {
    /// The type of room is not yet known since we're still connecting to it.
    Unknown,
    DirectMessage,
    Group,
    PrivateChannel,
    PublicChannel,
    Generic,
}

impl Display for RoomType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RoomType::Unknown => write!(f, "Unknown"),
            RoomType::DirectMessage => write!(f, "Direct Message"),
            RoomType::Group => write!(f, "Group"),
            RoomType::PrivateChannel => write!(f, "Private Channel"),
            RoomType::PublicChannel => write!(f, "Public Channel"),
            RoomType::Generic => write!(f, "Generic"),
        }
    }
}

impl From<BookmarkType> for RoomType {
    fn from(value: BookmarkType) -> Self {
        match value {
            BookmarkType::DirectMessage => RoomType::DirectMessage,
            BookmarkType::Group => RoomType::Group,
            BookmarkType::PrivateChannel => RoomType::PrivateChannel,
            BookmarkType::PublicChannel => RoomType::PublicChannel,
            BookmarkType::Generic => RoomType::Generic,
        }
    }
}
