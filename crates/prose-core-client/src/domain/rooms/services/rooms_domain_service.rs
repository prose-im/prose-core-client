// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::rooms::models::{RoomError, RoomInternals, RoomSpec};
use crate::domain::shared::models::{RoomId, UserId};

#[derive(Debug, Clone, PartialEq)]
pub enum CreateRoomType {
    Group { participants: Vec<UserId> },
    PrivateChannel { name: String },
    PublicChannel { name: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreateOrEnterRoomRequest {
    Create {
        service: BareJid,
        room_type: CreateRoomType,
    },
    JoinRoom {
        room_jid: RoomId,
        password: Option<String>,
    },
    JoinDirectMessage {
        participant: UserId,
    },
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait RoomsDomainService: SendUnlessWasm + SyncUnlessWasm {
    async fn create_or_join_room(
        &self,
        request: CreateOrEnterRoomRequest,
    ) -> Result<Arc<RoomInternals>, RoomError>;

    /// Renames the room identified by `room_jid` to `name`.
    ///
    /// If the room is not connected no action is performed, otherwise:
    /// - Panics if the Room is not of type `RoomType::PublicChannel`, `RoomType::PrivateChannel`
    ///   or `RoomType::Generic`.
    /// - Fails with `RoomError::PublicChannelNameConflict` if the room is of type
    ///   `RoomType::PublicChannel` and `name` is already used by another public channel.
    /// - Dispatches `ClientEvent::RoomChanged` of type `RoomEventType::AttributesChanged`
    ///   after processing.
    async fn rename_room(&self, room_jid: &RoomId, name: &str) -> Result<(), RoomError>;

    /// Reconfigures the room identified by `room_jid` according to `spec` and renames it to `new_name`.
    ///
    /// If the room is not connected no action is performed, otherwise:
    /// - Panics if the reconfiguration is not not allowed. Allowed reconfigurations are:
    ///   - `RoomType::Group` -> `RoomType::PrivateChannel`
    ///   - `RoomType::PublicChannel` -> `RoomType::PrivateChannel`
    ///   - `RoomType::PrivateChannel` -> `RoomType::PublicChannel`
    /// - Dispatches `ClientEvent::RoomChanged` of type `RoomEventType::AttributesChanged`
    ///   after processing.
    async fn reconfigure_room_with_spec(
        &self,
        room_jid: &RoomId,
        spec: RoomSpec,
        new_name: &str,
    ) -> Result<Arc<RoomInternals>, RoomError>;
}
