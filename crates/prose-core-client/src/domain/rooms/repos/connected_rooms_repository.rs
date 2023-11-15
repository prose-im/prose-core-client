// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::rooms::models::RoomInternals;
use crate::domain::shared::models::RoomJid;

type UpdateHandler = Box<dyn FnOnce(Arc<RoomInternals>) -> RoomInternals + Send>;

pub struct RoomAlreadyExistsError;

#[cfg_attr(feature = "test", mockall::automock)]
pub trait ConnectedRoomsReadOnlyRepository: SendUnlessWasm + SyncUnlessWasm {
    fn get(&self, room_jid: &RoomJid) -> Option<Arc<RoomInternals>>;
    fn get_all(&self) -> Vec<Arc<RoomInternals>>;
}

pub trait ConnectedRoomsRepository: ConnectedRoomsReadOnlyRepository {
    fn set(&self, room: Arc<RoomInternals>) -> Result<(), RoomAlreadyExistsError>;

    /// If a room with `room_jid` was found returns the room returned by `block` otherwise
    /// returns `None`.
    fn update(&self, room_jid: &RoomJid, block: UpdateHandler) -> Option<Arc<RoomInternals>>;

    fn delete(&self, room_jid: &RoomJid);
    fn delete_all(&self);
}

#[cfg(feature = "test")]
mockall::mock! {
    pub ConnectedRoomsReadWriteRepository {}

    impl ConnectedRoomsReadOnlyRepository for ConnectedRoomsReadWriteRepository {
        fn get(&self, room_jid: &RoomJid) -> Option<Arc<RoomInternals>>;
        fn get_all(&self) -> Vec<Arc<RoomInternals>>;
    }

    impl ConnectedRoomsRepository for ConnectedRoomsReadWriteRepository {
        fn set(&self, room: Arc<RoomInternals>) -> Result<(), RoomAlreadyExistsError>;
        fn update(&self, room_jid: &RoomJid, block: UpdateHandler) -> Option<Arc<RoomInternals>>;
        fn delete(&self, room_jid: &RoomJid);
        fn delete_all(&self);
    }
}
