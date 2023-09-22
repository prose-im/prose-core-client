// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::room;
use crate::room::{Generic, Room};

pub enum ConnectedRoom<D: DataCache + 'static, A: AvatarCache + 'static> {
    DirectMessage(Room<room::DirectMessage, D, A>),
    Group(Room<room::Group, D, A>),
    PrivateChannel(Room<room::PrivateChannel, D, A>),
    PublicChannel(Room<room::PublicChannel, D, A>),
    /// A generic MUC room that doesn't match any of our requirements
    Generic(Room<room::Generic, D, A>),
}

impl<D: DataCache, A: AvatarCache> ConnectedRoom<D, A> {
    pub fn to_generic_room(&self) -> Room<Generic, D, A> {
        match self {
            Self::DirectMessage(room) => room.to_base(),
            Self::Group(room) => room.to_base(),
            Self::PrivateChannel(room) => room.to_base(),
            Self::PublicChannel(room) => room.to_base(),
            Self::Generic(room) => room.to_base(),
        }
    }
}

impl<D: DataCache, A: AvatarCache> Clone for ConnectedRoom<D, A> {
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
