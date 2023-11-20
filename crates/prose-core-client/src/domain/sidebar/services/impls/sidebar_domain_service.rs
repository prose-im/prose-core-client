// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::{bail, Result};
use async_trait::async_trait;
use tracing::error;

use prose_proc_macros::DependenciesStruct;

use crate::app::deps::{
    DynBookmarksService, DynClientEventDispatcher, DynConnectedRoomsRepository,
    DynRoomManagementService, DynRoomsDomainService, DynSidebarRepository,
};
use crate::domain::rooms::models::RoomInternals;
use crate::domain::rooms::services::CreateOrEnterRoomRequest;
use crate::domain::shared::models::{RoomJid, RoomType};
use crate::domain::sidebar::models::{Bookmark, BookmarkType, SidebarItem};
use crate::ClientEvent;

use super::super::SidebarDomainService as SidebarDomainServiceTrait;

#[derive(DependenciesStruct)]
pub struct SidebarDomainService {
    bookmarks_service: DynBookmarksService,
    client_event_dispatcher: DynClientEventDispatcher,
    connected_rooms_repo: DynConnectedRoomsRepository,
    room_management_service: DynRoomManagementService,
    rooms_domain_service: DynRoomsDomainService,
    sidebar_repo: DynSidebarRepository,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl SidebarDomainServiceTrait for SidebarDomainService {
    /// Extends the sidebar with items by loading bookmarks from the remote PubSub node.
    ///
    /// Loads the remote bookmarks then proceeds with the logic details
    /// in `extend_items_from_bookmarks`.
    async fn load_and_extend_items_from_bookmarks(&self) -> Result<()> {
        let bookmarks = self.bookmarks_service.load_bookmarks().await?;
        self.extend_items_from_bookmarks(bookmarks).await?;
        Ok(())
    }

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
    async fn extend_items_from_bookmarks(&self, bookmarks: Vec<Bookmark>) -> Result<()> {
        let mut sidebar_changed = false;

        for bookmark in bookmarks {
            if let Some(mut sidebar_item) = self.sidebar_repo.get(&bookmark.jid) {
                // Update basic properties
                sidebar_item.name = bookmark.name;
                sidebar_item.is_favorite = bookmark.is_favorite;
                sidebar_item.r#type = bookmark.r#type;
                sidebar_changed = true;

                // The bookmark was removed from the sidebar. This can happen with Groups or
                // Private Channels, as Private Channels are kept in the bookmarks list because
                // we'd otherwise loose track of them, while Groups are kept because these should
                // always be connected so that our user can receive messages from them.
                if !bookmark.in_sidebar {
                    self.disconnect_room_for_removed_sidebar_item_if_needed(&sidebar_item)
                        .await?;
                    self.sidebar_repo.delete(&sidebar_item.jid);
                } else {
                    self.sidebar_repo.put(&sidebar_item);
                }

                continue;
            }

            if !bookmark.in_sidebar {
                continue;
            }

            let join_result = self
                .join_room_identified_by_bookmark_if_needed(&bookmark)
                .await;
            sidebar_changed = true;

            let sidebar_item = match join_result {
                Ok(None) => continue,
                Ok(Some(room)) => SidebarItem {
                    name: room.name().unwrap_or(bookmark.name),
                    jid: bookmark.jid,
                    r#type: bookmark.r#type,
                    is_favorite: bookmark.is_favorite,
                    error: None,
                },
                Err(err) => SidebarItem {
                    name: bookmark.name,
                    jid: bookmark.jid,
                    r#type: bookmark.r#type,
                    is_favorite: bookmark.is_favorite,
                    error: Some(err.to_string()),
                },
            };

            self.sidebar_repo.put(&sidebar_item);
        }

        if sidebar_changed {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::SidebarChanged);
        }

        Ok(())
    }

    /// Inserts a sidebar item by creating or joining a room based on the specified request.
    ///
    /// - If the room already exists in the sidebar, it returns the existing item.
    /// - For a new or joined room, it creates a new sidebar item.
    /// - Saves a bookmark for the new or joined room.
    /// - Dispatches a `ClientEvent::SidebarChanged` event after processing.
    async fn insert_item_by_creating_or_joining_room(
        &self,
        request: CreateOrEnterRoomRequest,
    ) -> Result<RoomJid> {
        let room = self
            .rooms_domain_service
            .create_or_join_room(request)
            .await?;

        if let Some(sidebar_item) = self.sidebar_repo.get(&room.jid) {
            return Ok(sidebar_item.jid);
        }

        let room_name = room.name().unwrap_or(room.jid.to_string());

        let bookmark_type = match room.r#type {
            RoomType::Pending => {
                unreachable!("RoomsDomainService unexpectedly returned a pending room.")
            }
            RoomType::DirectMessage => BookmarkType::DirectMessage,
            RoomType::Group => BookmarkType::Group,
            RoomType::PrivateChannel => BookmarkType::PrivateChannel,
            RoomType::PublicChannel => BookmarkType::PublicChannel,
            RoomType::Generic => {
                bail!("The joined/created room did not match any of our specifications.")
            }
        };

        let result = self
            .bookmarks_service
            .save_bookmark(&Bookmark {
                name: room_name.clone(),
                jid: room.jid.clone(),
                r#type: bookmark_type.clone(),
                is_favorite: false,
                in_sidebar: true,
            })
            .await;

        match result {
            Ok(_) => (),
            Err(error) => {
                error!("Failed to save bookmark for room {}. {}", room.jid, error)
            }
        }

        let sidebar_item = SidebarItem {
            name: room_name,
            jid: room.jid.clone(),
            r#type: bookmark_type.clone(),
            is_favorite: false,
            error: None,
        };

        self.sidebar_repo.put(&sidebar_item);

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(sidebar_item.jid)
    }

    /// Ensures a sidebar item exists for an active direct message or group conversation.
    ///
    /// If a message is received from a direct message or group that is not currently represented
    /// in the sidebar, this method will insert an item into the sidebar and update the
    /// corresponding bookmark.
    ///
    /// Dispatches a `ClientEvent::SidebarChanged` event after processing.
    async fn insert_item_for_received_message_if_needed(&self, room_jid: &RoomJid) -> Result<()> {
        // We do not need to create or join rooms here since we couldn't have received a message
        // from a room we're not connected to. Also rooms for groups are always connected no matter
        // if they are in the sidebar or not.
        let Some(room) = self.connected_rooms_repo.get(room_jid) else {
            return Ok(());
        };

        let bookmark_type = match room.r#type {
            RoomType::DirectMessage => BookmarkType::DirectMessage,
            RoomType::Group => BookmarkType::Group,
            _ => return Ok(()),
        };

        if self.sidebar_repo.get(&room.jid).is_some() {
            return Ok(());
        }

        let bookmark_name = room.name().unwrap_or("Untitled Conversation".to_string());

        self.bookmarks_service
            .save_bookmark(&Bookmark {
                name: bookmark_name.clone(),
                jid: room.jid.clone(),
                r#type: bookmark_type.clone(),
                is_favorite: false,
                in_sidebar: true,
            })
            .await?;

        self.sidebar_repo.put(&SidebarItem {
            name: bookmark_name,
            jid: room.jid.clone(),
            r#type: bookmark_type,
            is_favorite: false,
            error: None,
        });

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }

    /// Renames the sidebar item identified by `room_jid` to `name`.
    ///
    /// If the item is not in the list of sidebar items no action is performed, otherwise:
    ///   - The corresponding room will be renamed.
    ///   - The corresponding bookmark will be renamed.
    ///   - `ClientEvent::SidebarChanged` will be dispatched after processing.
    async fn rename_item(&self, room_jid: &RoomJid, name: &str) -> Result<()> {
        // If we don't have a sidebar item for this room there's no point in renaming it. It would
        // either not be connected or be a group which cannot be renamed.
        let Some(mut item) = self.sidebar_repo.get(room_jid) else {
            return Ok(());
        };

        // Nothing changed.
        if item.name.to_lowercase() == name.to_lowercase() {
            return Ok(());
        }

        // Rename the room first. If that succeeds continue.
        self.rooms_domain_service
            .rename_room(room_jid, name)
            .await?;

        item.name = name.to_string();
        self.sidebar_repo.put(&item);

        self.bookmarks_service
            .save_bookmark(&Bookmark::from(&item))
            .await?;

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }

    /// Toggles the `is_favorite` flag for the sidebar item identified by `room_jid`.
    ///
    /// If the item is not in the list of sidebar items no action is performed, otherwise:
    ///   - The corresponding bookmark will be updated to reflect the new status of `is_favorite`.
    ///   - `ClientEvent::SidebarChanged` will be dispatched after processing.
    async fn toggle_item_is_favorite(&self, room_jid: &RoomJid) -> Result<()> {
        let Some(mut sidebar_item) = self.sidebar_repo.get(room_jid) else {
            return Ok(());
        };

        sidebar_item.is_favorite ^= true;

        self.bookmarks_service
            .save_bookmark(&Bookmark::from(&sidebar_item))
            .await?;
        self.sidebar_repo.put(&sidebar_item);

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }

    /// Removes multiple sidebar items associated with the provided `room_jids`.
    ///
    /// - Disconnects channels and updates the repository state for each provided JID.
    /// - Groups and Private Channels have their bookmarks updated to reflect they are not in
    ///   the sidebar.
    /// - DirectMessages and Public Channels are deleted from bookmarks, as they do not require
    ///   persistent connections and can be rediscovered.
    /// - Triggers a `ClientEvent::SidebarChanged` event after processing to notify of the
    ///   sidebar update.
    async fn remove_items(&self, room_jids: &[&RoomJid]) -> Result<()> {
        for jid in room_jids {
            self.remove_item(*jid).await?;
        }

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }

    /// Handles remote deletion of bookmarks.
    ///
    /// - Disconnects channels and updates the repository state for each provided JID.
    /// - Bookmarks remain untouched.
    /// - Dispatches a `ClientEvent::SidebarChanged` event after processing.
    async fn handle_removed_items(&self, room_jids: &[&RoomJid]) -> Result<()> {
        for jid in room_jids {
            let Some(sidebar_item) = self.sidebar_repo.get(jid) else {
                continue;
            };
            self.disconnect_room_for_removed_sidebar_item_if_needed(&sidebar_item)
                .await?;
            self.sidebar_repo.delete(&jid);
        }

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }

    /// Disconnects *all* rooms and deletes all sidebar items. Dispatches
    /// a `ClientEvent::SidebarChanged` event after processing.
    ///
    /// This method exists to handle the (rare) case where our bookmarks PubSub node is either
    /// purged or deleted altogether. It should usually only happen when debugging.
    async fn handle_remote_purge(&self) -> Result<()> {
        // We're not iterating over the sidebar items here since the connected_rooms_repo might
        // contain rooms that we don't have sidebar items for (like Groups that are not currently
        // visible in the sidebar). So we disconnect all rooms and delete all sidebar
        // items afterwards.
        for room in self.connected_rooms_repo.get_all() {
            if room.r#type == RoomType::DirectMessage {
                continue;
            }
            let full_jid = room.user_full_jid();
            self.room_management_service.exit_room(&full_jid).await?;
        }

        // No need to delete the bookmarks here since that is the raison d'etre for this method.
        self.connected_rooms_repo.delete_all();
        self.sidebar_repo.delete_all();

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }

    /// Removes all connected rooms and sidebar items.
    ///
    /// Call this method after logging out.
    async fn clear_cache(&self) -> Result<()> {
        self.sidebar_repo.delete_all();
        self.connected_rooms_repo.delete_all();
        Ok(())
    }
}

impl SidebarDomainService {
    /// Removes a sidebar item associated with the given `jid` and updates the room's
    /// connection status.
    ///
    /// - If the sidebar item exists, it attempts to disconnect the room if it's a channel.
    /// - For Groups and Private Channels:
    ///   - The bookmarks are updated to reflect they are not favorites and not in the sidebar.
    ///   - Groups remain connected to ensure message receipt.
    ///   - Private Channels are kept tracked to avoid losing them as they are not discoverable
    ///     through the MUC service.
    /// - DirectMessages and Public Channels:
    ///   - The bookmarks are fully deleted as DirectMessages do not need to be tracked and
    ///     Public Channels can be rediscovered.
    /// - The `SidebarRepository` is updated by removing the item.
    async fn remove_item(&self, jid: &RoomJid) -> Result<()> {
        let Some(sidebar_item) = self.sidebar_repo.get(jid) else {
            return Ok(());
        };

        self.disconnect_room_for_removed_sidebar_item_if_needed(&sidebar_item)
            .await?;

        match sidebar_item.r#type {
            // For Groups and Private Channels we do not really delete the bookmarks. The reason
            // is that Groups should always be connected so that our user can receive messages from
            // them, while we keep references to the Private channels because we'd otherwise loose
            // track of them since the MUC service at this time only let's us discover
            // public channels.
            BookmarkType::Group | BookmarkType::PrivateChannel => {
                let mut bookmark = Bookmark::from(&sidebar_item);
                bookmark.is_favorite = false;
                bookmark.in_sidebar = false;
                self.bookmarks_service.save_bookmark(&bookmark).await?;
            }
            BookmarkType::DirectMessage | BookmarkType::PublicChannel => {
                self.bookmarks_service.delete_bookmark(&jid).await?;
            }
        }

        self.sidebar_repo.delete(&jid);
        Ok(())
    }

    /// Disconnects from a room associated with a removed sidebar item if necessary.
    ///
    /// - DirectMessages are not disconnected as they are not MUC rooms and do not require
    ///   disconnection.
    /// - Groups, being MUC rooms, should remain connected to ensure users always receive messages
    ///   and are not disconnected.
    /// - Private and Public Channels are disconnected when they are removed from the sidebar.
    ///
    /// If a room is disconnected, it is removed from the `ConnectedRoomsRepository`.
    async fn disconnect_room_for_removed_sidebar_item_if_needed(
        &self,
        sidebar_item: &SidebarItem,
    ) -> Result<()> {
        match sidebar_item.r#type {
            // DirectMessages do not need to be connected as they are not MUC rooms
            BookmarkType::DirectMessage => return Ok(()),
            // Groups will always be connected so that they behave like DirectMessages insofar that
            // our user should always receive messages from them.
            BookmarkType::Group => return Ok(()),
            // Private and Public Channels actually will be disconnected.
            BookmarkType::PrivateChannel | BookmarkType::PublicChannel => {
                if let Some(room) = self.connected_rooms_repo.get(&sidebar_item.jid) {
                    let full_jid = room.user_full_jid();
                    self.room_management_service.exit_room(&full_jid).await?;
                }
            }
        }

        Ok(())
    }

    /// Attempts to join a room based on the given `bookmark`.
    ///
    /// - DirectMessage rooms are not joined as they are not MUC rooms.
    /// - Channels are only joined if they appear in the sidebar (`Bookmark::in_sidebar`).
    /// - Groups are always joined, regardless of sidebar status, to ensure message receipt.
    ///
    /// If a room is joined, it is added to the `ConnectedRoomsRepository`.
    async fn join_room_identified_by_bookmark_if_needed(
        &self,
        bookmark: &Bookmark,
    ) -> Result<Option<Arc<RoomInternals>>> {
        let room = match bookmark.r#type {
            BookmarkType::DirectMessage if !bookmark.in_sidebar => None,

            // For channels, we're only participating in them if they're in the sidebar.
            BookmarkType::PublicChannel | BookmarkType::PrivateChannel if !bookmark.in_sidebar => {
                None
            }

            // Since direct messages are not MUC rooms we don't need to connect to them. But we'll
            // insert the placeholder room instead.
            BookmarkType::DirectMessage => Some(
                self.rooms_domain_service
                    .create_or_join_room(CreateOrEnterRoomRequest::JoinDirectMessage {
                        participant: bookmark.jid.clone().into_inner(),
                    })
                    .await?,
            ),

            // While our user can remove a Group from their sidebar they should always receive
            // messages from it. In these cases the Group will automatically reappear in the
            // sidebar. We want our users to think about Groups as if they were a
            // Direct Message.
            BookmarkType::Group | BookmarkType::PublicChannel | BookmarkType::PrivateChannel => {
                Some(
                    self.rooms_domain_service
                        .create_or_join_room(CreateOrEnterRoomRequest::JoinRoom {
                            room_jid: bookmark.jid.clone(),
                            password: None,
                        })
                        .await?,
                )
            }
        };

        Ok(room)
    }
}

impl From<&SidebarItem> for Bookmark {
    fn from(value: &SidebarItem) -> Self {
        Self {
            name: value.name.clone(),
            jid: value.jid.clone(),
            r#type: value.r#type.clone(),
            is_favorite: value.is_favorite,
            in_sidebar: true,
        }
    }
}
