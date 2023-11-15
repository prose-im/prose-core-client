// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use connected_rooms_repository::{
    ConnectedRoomsReadOnlyRepository, ConnectedRoomsRepository, RoomAlreadyExistsError,
};

mod connected_rooms_repository;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::connected_rooms_repository::MockConnectedRoomsReadOnlyRepository;
    pub use super::connected_rooms_repository::MockConnectedRoomsReadWriteRepository;
}
