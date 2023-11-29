// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use jid::{BareJid, FullJid};

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::rooms::models::{PublicRoomInfo, RoomError, RoomSessionInfo, RoomSpec};
use crate::dtos::RoomId;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait RoomManagementService: SendUnlessWasm + SyncUnlessWasm {
    async fn load_public_rooms(
        &self,
        muc_service: &BareJid,
    ) -> Result<Vec<PublicRoomInfo>, RoomError>;

    async fn create_or_join_room(
        &self,
        room_jid: &FullJid,
        room_name: &str,
        spec: RoomSpec,
    ) -> Result<RoomSessionInfo, RoomError>;

    async fn join_room(
        &self,
        room_jid: &FullJid,
        password: Option<&str>,
    ) -> Result<RoomSessionInfo, RoomError>;

    async fn reconfigure_room(
        &self,
        room_jid: &RoomId,
        spec: RoomSpec,
        new_name: &str,
    ) -> Result<(), RoomError>;

    async fn exit_room(&self, room_jid: &FullJid) -> Result<(), RoomError>;

    async fn set_room_owners(&self, room_jid: &RoomId, users: &[BareJid]) -> Result<(), RoomError>;

    /// Destroys the room identified by `room_jid`. If specified sets `alternate_room` as
    /// replacement room, so that users will be redirected there.
    async fn destroy_room(
        &self,
        room_jid: &RoomId,
        alternate_room: Option<RoomId>,
    ) -> Result<(), RoomError>;
}
