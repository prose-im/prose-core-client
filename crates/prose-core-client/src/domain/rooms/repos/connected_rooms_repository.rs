// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::BareJid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::rooms::models::Room;
use crate::domain::shared::models::AccountId;

type UpdateHandler = Box<dyn FnOnce(Room) -> Room + Send>;

pub struct RoomAlreadyExistsError;

#[cfg_attr(feature = "test", mockall::automock)]
pub trait ConnectedRoomsReadOnlyRepository: SendUnlessWasm + SyncUnlessWasm {
    fn get(&self, account: &AccountId, room_id: &BareJid) -> Option<Room>;
    fn get_all(&self, account: &AccountId) -> Vec<Room>;
}

pub trait ConnectedRoomsRepository: ConnectedRoomsReadOnlyRepository {
    fn set(&self, account: &AccountId, room: Room) -> Result<(), RoomAlreadyExistsError>;

    fn set_or_replace(&self, account: &AccountId, room: Room) -> Option<Room>;

    /// If a room with `room_id` was found returns the room returned by `block` otherwise
    /// returns `None`.
    fn update(&self, account: &AccountId, room_id: &BareJid, block: UpdateHandler) -> Option<Room>;

    /// Deletes the room identified by `room_id` from the repository and returns the removed room.
    fn delete(&self, account: &AccountId, room_id: &BareJid) -> Option<Room>;

    /// Deletes all rooms from the repository and returns the removed rooms.
    fn delete_all(&self, account: &AccountId) -> Vec<Room>;
}

#[cfg(feature = "test")]
mockall::mock! {
    pub ConnectedRoomsReadWriteRepository {}

    impl ConnectedRoomsReadOnlyRepository for ConnectedRoomsReadWriteRepository {
        fn get(&self, account: &AccountId, room_id: &BareJid) -> Option<Room>;
        fn get_all(&self, account: &AccountId) -> Vec<Room>;
    }

    impl ConnectedRoomsRepository for ConnectedRoomsReadWriteRepository {
        fn set(&self, account: &AccountId, room: Room) -> Result<(), RoomAlreadyExistsError>;
        fn set_or_replace(&self, account: &AccountId, room: Room) -> Option<Room>;
        fn update(&self, account: &AccountId, room_id: &BareJid, block: UpdateHandler) -> Option<Room>;
        fn delete(&self, account: &AccountId, room_id: &BareJid) -> Option<Room>;
        fn delete_all(&self, account: &AccountId) -> Vec<Room>;
    }
}
