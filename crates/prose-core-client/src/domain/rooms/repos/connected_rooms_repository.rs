// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::rooms::models::Room;
use crate::domain::shared::models::RoomId;

type UpdateHandler = Box<dyn FnOnce(Room) -> Room + Send>;

pub struct RoomAlreadyExistsError;

#[cfg_attr(feature = "test", mockall::automock)]
pub trait ConnectedRoomsReadOnlyRepository: SendUnlessWasm + SyncUnlessWasm {
    fn get(&self, room_id: &RoomId) -> Option<Room>;
    fn get_all(&self) -> Vec<Room>;
}

pub trait ConnectedRoomsRepository: ConnectedRoomsReadOnlyRepository {
    fn set(&self, room: Room) -> Result<(), RoomAlreadyExistsError>;

    fn set_or_replace(&self, room: Room) -> Option<Room>;

    /// If a room with `room_id` was found returns the room returned by `block` otherwise
    /// returns `None`.
    fn update(&self, room_id: &RoomId, block: UpdateHandler) -> Option<Room>;

    /// Deletes the room identified by `room_id` from the repository and returns the removed room.
    fn delete(&self, room_id: &RoomId) -> Option<Room>;

    /// Deletes all rooms from the repository and returns the removed rooms.
    fn delete_all(&self) -> Vec<Room>;
}

#[cfg(feature = "test")]
mockall::mock! {
    pub ConnectedRoomsReadWriteRepository {}

    impl ConnectedRoomsReadOnlyRepository for ConnectedRoomsReadWriteRepository {
        fn get(&self, room_id: &RoomId) -> Option<Room>;
        fn get_all(&self) -> Vec<Room>;
    }

    impl ConnectedRoomsRepository for ConnectedRoomsReadWriteRepository {
        fn set(&self, room: Room) -> Result<(), RoomAlreadyExistsError>;
        fn set_or_replace(&self, room: Room) -> Option<Room>;
        fn update(&self, room_id: &RoomId, block: UpdateHandler) -> Option<Room>;
        fn delete(&self, room_id: &RoomId) -> Option<Room>;
        fn delete_all(&self) -> Vec<Room>;
    }
}
