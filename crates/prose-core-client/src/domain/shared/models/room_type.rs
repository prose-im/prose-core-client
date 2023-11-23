// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum RoomType {
    Pending,
    DirectMessage,
    Group,
    PrivateChannel,
    PublicChannel,
    Generic,
}

impl Display for RoomType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RoomType::Pending => write!(f, "Pending"),
            RoomType::DirectMessage => write!(f, "Direct Message"),
            RoomType::Group => write!(f, "Group"),
            RoomType::PrivateChannel => write!(f, "Private Channel"),
            RoomType::PublicChannel => write!(f, "Public Channel"),
            RoomType::Generic => write!(f, "Generic"),
        }
    }
}
