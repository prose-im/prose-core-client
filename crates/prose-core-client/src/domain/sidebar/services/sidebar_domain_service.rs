// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::rooms::services::CreateOrEnterRoomRequest;
use crate::domain::shared::models::RoomJid;
use crate::domain::sidebar::models::Bookmark;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait SidebarDomainService: SendUnlessWasm + SyncUnlessWasm {
    /// Extends the sidebar with items by loading bookmarks from the remote PubSub node.
    ///
    /// Loads the remote bookmarks then proceeds with the logic details
    /// in `extend_items_from_bookmarks`.
    async fn load_and_extend_items_from_bookmarks(&self) -> Result<()>;

    /// Extends the sidebar with items from a collection of bookmarks.
    ///
    /// Iterates through the provided bookmarks and performs the following actions:
    /// - If a sidebar item exists for a bookmark, it updates the item with the
    ///   bookmark's properties.
    /// - If the bookmark is no longer in the sidebar, it attempts to disconnect the
    ///   associated room.
    /// - If no sidebar item exists, it tries to join the room identified by the bookmark.
    ///   - On success, a new sidebar item is created with the room's details or the bookmark's
    ///     details if the room has no name.
    ///   - On failure, a new sidebar item is created with an error state.
    ///
    /// After processing all bookmarks, dispatches a `ClientEvent::SidebarChanged`.
    async fn extend_items_from_bookmarks(&self, bookmarks: Vec<Bookmark>) -> Result<()>;

    /// Inserts a sidebar item by creating or joining a room based on the specified request.
    ///
    /// - If the room already exists in the sidebar, it returns the existing item.
    /// - For a new or joined room, it creates a new sidebar item.
    /// - Saves a bookmark for the new or joined room.
    /// - Dispatches a `ClientEvent::SidebarChanged` event after processing.
    async fn insert_item_by_creating_or_joining_room(
        &self,
        request: CreateOrEnterRoomRequest,
    ) -> Result<RoomJid>;

    /// Ensures a sidebar item exists for an active direct message or group conversation.
    ///
    /// If a message is received from a direct message or group that is not currently represented
    /// in the sidebar, this method will insert an item into the sidebar and update the
    /// corresponding bookmark.
    ///
    /// Dispatches a `ClientEvent::SidebarChanged` event after processing.
    async fn insert_item_for_received_message_if_needed(&self, room_jid: &RoomJid) -> Result<()>;

    /// Renames the sidebar item identified by `room_jid` to `name`.
    ///
    /// If the item is not in the list of sidebar items no action is performed, otherwise:
    ///   - The corresponding room will be renamed.
    ///   - The corresponding bookmark will be renamed.
    ///   - `ClientEvent::SidebarChanged` will be dispatched after processing.
    async fn rename_item(&self, room_jid: &RoomJid, name: &str) -> Result<()>;

    /// Toggles the `is_favorite` flag for the sidebar item identified by `room_jid`.
    ///
    /// If the item is not in the list of sidebar items no action is performed, otherwise:
    ///   - The corresponding bookmark will be updated to reflect the new status of `is_favorite`.
    ///   - `ClientEvent::SidebarChanged` will be dispatched after processing.
    async fn toggle_item_is_favorite(&self, room_jid: &RoomJid) -> Result<()>;

    /// Removes multiple sidebar items associated with the provided `room_jids`.
    ///
    /// - Disconnects channels and updates the repository state for each provided JID.
    /// - Groups and Private Channels have their bookmarks updated to reflect they are not in
    ///   the sidebar.
    /// - DirectMessages and Public Channels are deleted from bookmarks, as they do not require
    ///   persistent connections and can be rediscovered.
    /// - Dispatches a `ClientEvent::SidebarChanged` event after processing.
    async fn remove_items(&self, room_jids: &[&RoomJid]) -> Result<()>;

    /// Handles remote deletion of bookmarks.
    ///
    /// - Disconnects channels and updates the repository state for each provided JID.
    /// - Bookmarks remain untouched.
    /// - Dispatches a `ClientEvent::SidebarChanged` event after processing.
    async fn handle_removed_items(&self, room_jids: &[&RoomJid]) -> Result<()>;

    /// Disconnects *all* rooms and deletes all sidebar items. Dispatches
    /// a `ClientEvent::SidebarChanged` event after processing.
    ///
    /// This method exists to handle the (rare) case where our bookmarks PubSub node is either
    /// purged or deleted altogether. It should usually only happen when debugging.
    async fn handle_remote_purge(&self) -> Result<()>;

    /// Removes all connected rooms and sidebar items.
    ///
    /// Call this method after logging out.
    async fn clear_cache(&self) -> Result<()>;
}