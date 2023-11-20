// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::RoomType;
use strum_macros::EnumIter;

/// Describes how a MUC room should be configured in order to match our idea of different
/// room types.
#[derive(Debug, Clone, EnumIter)]
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
}
