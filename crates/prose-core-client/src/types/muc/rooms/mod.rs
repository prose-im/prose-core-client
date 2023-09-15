// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub(super) use abstract_room::AbstractRoom;

mod abstract_room;

#[derive(Debug, Clone)]
pub struct Group {
    pub(super) room: AbstractRoom,
}

#[derive(Debug, Clone)]
pub struct PrivateChannel {
    pub(super) room: AbstractRoom,
}

#[derive(Debug, Clone)]
pub struct PublicChannel {
    pub(super) room: AbstractRoom,
}

#[derive(Debug, Clone)]
pub struct GenericRoom {
    pub(super) room: AbstractRoom,
}
