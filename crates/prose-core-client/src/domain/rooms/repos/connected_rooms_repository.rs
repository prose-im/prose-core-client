// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::rooms::models::RoomInternals;
use crate::domain::shared::models::RoomJid;

type UpdateHandler = Box<dyn FnOnce(Arc<RoomInternals>) -> RoomInternals + Send>;

pub struct RoomAlreadyExistsError;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait ConnectedRoomsRepository: SendUnlessWasm + SyncUnlessWasm {
    fn get(&self, room_jid: &RoomJid) -> Option<Arc<RoomInternals>>;
    fn get_all(&self) -> Vec<Arc<RoomInternals>>;

    fn set(&self, room: Arc<RoomInternals>) -> Result<(), RoomAlreadyExistsError>;

    /// Replaces all rooms with `rooms`.
    fn replace(&self, rooms: Vec<RoomInternals>);

    /// If a room with `room_jid` was found returns the room returned by `block` otherwise
    /// returns `None`.
    fn update(&self, room_jid: &RoomJid, block: UpdateHandler) -> Option<Arc<RoomInternals>>;

    fn delete<'a>(&self, room_jids: &[&'a RoomJid]);

    async fn clear_cache(&self) -> Result<()>;
}
