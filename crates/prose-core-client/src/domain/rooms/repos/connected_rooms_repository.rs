// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::rooms::models::RoomInternals;

type UpdateHandler = Box<dyn FnOnce(Arc<RoomInternals>) -> RoomInternals + Send>;

pub struct RoomAlreadyExistsError;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait ConnectedRoomsRepository: SendUnlessWasm + SyncUnlessWasm {
    fn get(&self, room_jid: &BareJid) -> Option<Arc<RoomInternals>>;
    fn get_all(&self) -> Vec<Arc<RoomInternals>>;

    fn set(&self, room: RoomInternals) -> Result<(), RoomAlreadyExistsError>;

    /// Replaces all rooms with `rooms`.
    fn replace(&self, rooms: Vec<RoomInternals>);

    /// If a room with `room_jid` was found returns the room returned by `block` otherwise
    /// returns `None`.
    fn update(&self, room_jid: &BareJid, block: UpdateHandler) -> Option<Arc<RoomInternals>>;

    fn delete<'a>(&self, room_jids: &[&'a BareJid]);

    async fn clear_cache(&self) -> Result<()>;
}
