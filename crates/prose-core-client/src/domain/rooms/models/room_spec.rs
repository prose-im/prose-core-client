// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

use strum_macros::EnumIter;

use crate::domain::shared::models::RoomType;
use crate::domain::sidebar::models::BookmarkType;

/// Describes how a MUC room should be configured in order to match our idea of different
/// room types.
#[derive(Debug, Clone, PartialEq, EnumIter)]
pub enum RoomSpec {
    // The order needs to be from most restrictive to least restrictive in order to correctly
    // identify a room type from a set of MUC properties.
    Group,
    PrivateChannel,
    PublicChannel,
}

impl RoomSpec {
    /// The type of room this spec creates…
    pub fn room_type(&self) -> RoomType {
        match self {
            RoomSpec::Group => RoomType::Group,
            RoomSpec::PrivateChannel => RoomType::PrivateChannel,
            RoomSpec::PublicChannel => RoomType::PublicChannel,
        }
    }

    /// The type of bookmark that matches this spec…
    pub fn bookmark_type(&self) -> BookmarkType {
        match self {
            RoomSpec::Group => BookmarkType::Group,
            RoomSpec::PrivateChannel => BookmarkType::PrivateChannel,
            RoomSpec::PublicChannel => BookmarkType::PublicChannel,
        }
    }
}

impl Display for RoomSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RoomSpec::Group => "Group",
                RoomSpec::PrivateChannel => "Private Channel",
                RoomSpec::PublicChannel => "Public Channel",
            }
        )
    }
}
