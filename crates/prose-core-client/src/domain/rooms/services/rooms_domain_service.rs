// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::encryption::models::DecryptionContext;
use crate::domain::rooms::models::{Room, RoomError, RoomSidebarState, RoomSpec};
use crate::domain::shared::models::{MucId, UserId};

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
        behavior: CreateRoomBehavior,
        decryption_context: Option<DecryptionContext>,
    },
    JoinRoom {
        room_id: MucId,
        password: Option<String>,
        behavior: JoinRoomBehavior,
        decryption_context: Option<DecryptionContext>,
    },
    JoinDirectMessage {
        participant: UserId,
        decryption_context: Option<DecryptionContext>,
    },
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum CreateRoomBehavior {
    /// Joins the specified alternate room when encountering a tombstone with the same JID.
    FollowIfGone,
    /// Fails the room creation operation if a tombstone with the same JID is encountered.
    FailIfGone,
    /// Creates a new, unique room by appending a monotonically increasing suffix when a room
    /// when encountering a tombstone with the same JID.
    CreateUniqueIfGone,
    /// Tries to join the specified alternate room when encountering a tombstone with the same JID.
    /// If no alternate room is specified, it creates a new, unique room with a monotonically
    /// increasing suffix.
    FollowThenCreateUnique,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct JoinRoomBehavior {
    pub on_redirect: JoinRoomRedirectBehavior,
    pub on_failure: JoinRoomFailureBehavior,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum JoinRoomRedirectBehavior {
    /// Joins the specified alternate room when encountering a tombstone with the same JID.
    FollowIfGone,
    /// Fails the room creation operation if a tombstone with the same JID is encountered.
    FailIfGone,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum JoinRoomFailureBehavior {
    /// If the connection fails, the room is removed.
    RemoveOnError,
    /// If the connection fails, the room will be retained and its status property set
    /// to `Disconnected` with an error message.
    RetainOnError,
}

impl JoinRoomBehavior {
    /// Defines the standard behavior if the join was user initiated. In such cases we would
    /// usually show a dedicated UI and an alert if the operation fails.
    pub fn user_initiated() -> Self {
        Self {
            on_redirect: JoinRoomRedirectBehavior::FollowIfGone,
            on_failure: JoinRoomFailureBehavior::RemoveOnError,
        }
    }

    /// Defines the standard behavior if the join was system initiated. This could be the case
    /// i.e. when we've received changed bookmarks from the server or an invite from another user.
    /// In such cases we want the room that could not be connected to, to stick around in the
    /// sidebar since we don't have a dedicated UI.
    pub fn system_initiated() -> Self {
        Self {
            on_redirect: JoinRoomRedirectBehavior::FollowIfGone,
            on_failure: JoinRoomFailureBehavior::RetainOnError,
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait RoomsDomainService: SendUnlessWasm + SyncUnlessWasm {
    async fn create_or_join_room(
        &self,
        request: CreateOrEnterRoomRequest,
        sidebar_state: RoomSidebarState,
    ) -> Result<Room, RoomError>;

    /// Renames the room identified by `room_jid` to `name`.
    ///
    /// If the room is not connected no action is performed, otherwise:
    /// - Panics if the Room is not of type `RoomType::PublicChannel`, `RoomType::PrivateChannel`
    ///   or `RoomType::Generic`.
    /// - Fails with `RoomError::PublicChannelNameConflict` if the room is of type
    ///   `RoomType::PublicChannel` and `name` is already used by another public channel.
    /// - Dispatches `ClientEvent::RoomChanged` of type `RoomEventType::AttributesChanged`
    ///   after processing.
    async fn rename_room(&self, room_id: &MucId, name: &str) -> Result<(), RoomError>;

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
        room_id: &MucId,
        spec: RoomSpec,
        new_name: &str,
    ) -> Result<Room, RoomError>;

    /// Loads the configuration for `room_id` and updates the corresponding `RoomInternals`
    /// accordingly. Call this method after the room configuration changed.
    /// Returns `RoomError::RoomNotFound` if no room with `room_id` exists.
    async fn reevaluate_room_spec(&self, room_id: &MucId) -> Result<Room, RoomError>;
}
