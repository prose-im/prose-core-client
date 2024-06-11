// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use jid::BareJid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::RequestError;

use crate::domain::general::models::Capabilities;
use crate::domain::rooms::models::{
    PublicRoomInfo, RoomConfig, RoomError, RoomSessionInfo, RoomSpec,
};
use crate::domain::shared::models::{MucId, OccupantId, UserId};
use crate::dtos::Availability;

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
        occupant_id: &OccupantId,
        room_name: &str,
        spec: RoomSpec,
        capabilities: &Capabilities,
        availability: Availability,
    ) -> Result<RoomSessionInfo, RoomError>;

    async fn join_room(
        &self,
        occupant_id: &OccupantId,
        password: Option<&str>,
        capabilities: &Capabilities,
        availability: Availability,
    ) -> Result<RoomSessionInfo, RoomError>;

    async fn reconfigure_room(
        &self,
        room_id: &MucId,
        spec: RoomSpec,
        new_name: &str,
    ) -> Result<(), RoomError>;

    async fn load_room_config(&self, room_id: &MucId) -> Result<RoomConfig, RoomError>;

    async fn exit_room(&self, occupant_id: &OccupantId) -> Result<(), RoomError>;

    async fn set_room_owners(&self, room_id: &MucId, users: &[UserId]) -> Result<(), RoomError>;

    async fn send_self_ping(&self, occupant_id: &OccupantId) -> Result<(), RequestError>;

    /// Destroys the room identified by `room_id`. If specified sets `alternate_room` as
    /// replacement room, so that users will be redirected there.
    async fn destroy_room(
        &self,
        room_id: &MucId,
        alternate_room: Option<MucId>,
    ) -> Result<(), RoomError>;
}
