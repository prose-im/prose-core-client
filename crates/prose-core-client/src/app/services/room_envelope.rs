// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::room::{DirectMessage, Generic, Group, PrivateChannel, PublicChannel, Room};

#[derive(Debug)]
pub enum RoomEnvelope {
    DirectMessage(Room<DirectMessage>),
    Group(Room<Group>),
    PrivateChannel(Room<PrivateChannel>),
    PublicChannel(Room<PublicChannel>),
    /// A generic MUC room that doesn't match any of our requirements
    Generic(Room<Generic>),
}

impl RoomEnvelope {
    pub fn to_generic_room(&self) -> Room<Generic> {
        match self {
            Self::DirectMessage(room) => room.to_generic(),
            Self::Group(room) => room.to_generic(),
            Self::PrivateChannel(room) => room.to_generic(),
            Self::PublicChannel(room) => room.to_generic(),
            Self::Generic(room) => room.to_generic(),
        }
    }
}

impl Clone for RoomEnvelope {
    fn clone(&self) -> Self {
        match self {
            Self::DirectMessage(room) => Self::DirectMessage(room.clone()),
            Self::Group(room) => Self::Group(room.clone()),
            Self::PrivateChannel(room) => Self::PrivateChannel(room.clone()),
            Self::PublicChannel(room) => Self::PublicChannel(room.clone()),
            Self::Generic(room) => Self::Generic(room.clone()),
        }
    }
}
