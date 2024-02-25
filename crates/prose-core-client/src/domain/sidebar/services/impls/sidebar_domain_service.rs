// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{bail, format_err, Context, Result};
use async_trait::async_trait;
use futures::future::join_all;
use tracing::{error, info};

use prose_proc_macros::DependenciesStruct;
use prose_wasm_utils::ProseFutureExt;

use crate::app::deps::{
    DynAppContext, DynBookmarksService, DynClientEventDispatcher, DynConnectedRoomsRepository,
    DynRoomManagementService, DynRoomsDomainService,
};
use crate::domain::rooms::models::{Room, RoomError, RoomSidebarState, RoomSpec, RoomState};
use crate::domain::rooms::services::impls::build_nickname;
use crate::domain::rooms::services::{CreateOrEnterRoomRequest, JoinRoomBehavior};
use crate::domain::shared::models::{RoomId, RoomType, UserEndpointId, UserId};
use crate::domain::sidebar::models::{Bookmark, BookmarkType};
use crate::dtos::Availability;
use crate::ClientEvent;

use super::super::SidebarDomainService as SidebarDomainServiceTrait;

#[derive(DependenciesStruct)]
pub struct SidebarDomainService {
    bookmarks_service: DynBookmarksService,
    client_event_dispatcher: DynClientEventDispatcher,
    connected_rooms_repo: DynConnectedRoomsRepository,
    ctx: DynAppContext,
    room_management_service: DynRoomManagementService,
    rooms_domain_service: DynRoomsDomainService,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl SidebarDomainServiceTrait for SidebarDomainService {
    /// Extends the sidebar with items by loading bookmarks from the remote PubSub node.
    ///
    /// Loads the remote bookmarks then proceeds with the logic details
    /// in `extend_items_from_bookmarks`.
    #[tracing::instrument(skip(self))]
    async fn populate_sidebar(&self) -> Result<()> {
        let bookmarks = self.bookmarks_service.load_bookmarks().await?;
        debug_assert!(self.connected_rooms_repo.get_all().is_empty());
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
        // let mut delete_rooms_futures = vec![];
        let mut join_room_futures = vec![];
        let mut update_bookmarks_futures = vec![];

        let nickname = build_nickname(&self.ctx.connected_id()?.to_user_id());
        let rooms = self.connected_rooms_repo.get_all();
        let mut rooms_changed = false;

        // We don't need to diff here between our connected rooms and the received bookmarks.
        // We're already receiving the diff from the PubSub node. Only when `populate_sidebar` is
        // called we're receiving all bookmarks at once, but in that case we won't have any
        // connected rooms. We might however receive bookmarks that we have rooms for from the
        // PubSub node if the bookmarks changed.

        // Insert a pending room for each bookmark so that we're able to draw the sidebar
        // before each room is connected.
        for bookmark in &bookmarks {
            if rooms.iter().find(|r| r.room_id == bookmark.jid).is_some() {
                continue;
            }
            rooms_changed = true;

            match bookmark.r#type {
                // Groups are always connected…
                BookmarkType::DirectMessage
                | BookmarkType::PrivateChannel
                | BookmarkType::PublicChannel
                | BookmarkType::Generic
                    if bookmark.sidebar_state == RoomSidebarState::NotInSidebar =>
                {
                    ()
                }
                BookmarkType::DirectMessage => {
                    _ = self.connected_rooms_repo.set(Room::for_direct_message(
                        &UserId::from(bookmark.jid.clone().into_inner()),
                        &bookmark.name,
                        Availability::Unavailable,
                        bookmark.sidebar_state,
                    ));
                }
                BookmarkType::Group
                | BookmarkType::PrivateChannel
                | BookmarkType::PublicChannel
                | BookmarkType::Generic => {
                    _ = self
                        .connected_rooms_repo
                        .set(Room::pending(&bookmark, &nickname));
                }
            };
        }

        if rooms_changed {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::SidebarChanged);
            rooms_changed = false;
        }

        // Now collect the futures to connect each room…
        for bookmark in bookmarks {
            if let Some(room) = rooms.iter().find(|r| r.room_id == bookmark.jid) {
                if room.sidebar_state() != bookmark.sidebar_state {
                    // We have a room for that bookmark already, let's just update its sidebar_state…
                    room.set_sidebar_state(bookmark.sidebar_state);
                    rooms_changed = true;
                }
                continue;
            }

            join_room_futures.push(async move {
                let result = self
                    .join_room_identified_by_bookmark_if_needed(
                        &bookmark,
                        JoinRoomBehavior::system_initiated(),
                    )
                    .await;

                match &result {
                    Ok(Some(_)) => {
                        if bookmark.sidebar_state.is_in_sidebar() {
                            // Fire an event each time a room connects…
                            self.client_event_dispatcher
                                .dispatch_event(ClientEvent::SidebarChanged);
                        }
                    }
                    Ok(None) => (),
                    Err(_) => {
                        if bookmark.sidebar_state.is_in_sidebar() {
                            self.client_event_dispatcher
                                .dispatch_event(ClientEvent::SidebarChanged);
                        }
                    }
                }

                (bookmark, result)
            });
        }

        if rooms_changed {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::SidebarChanged);
        }

        // …and run them in parallel.
        let results: Vec<(Bookmark, Result<Option<Room>, RoomError>)> =
            join_all(join_room_futures).await;

        // Now evaluate the results…
        for (bookmark, result) in results {
            let room_id = bookmark.jid;
            match result {
                Ok(Some(room)) => {
                    // The room was gone and we followed the redirect…
                    if room.room_id != room_id {
                        update_bookmarks_futures.push(
                            async move {
                                self.save_bookmark_for_room(&room).await;
                                self.delete_bookmark(&room_id).await;
                            }
                            .prose_boxed(),
                        );
                        continue;
                    }

                    let bookmark_type = BookmarkType::from(room.r#type);

                    // The room has different attributes than the ones saved in our bookmark…
                    if bookmark.r#type != bookmark_type
                        || Some(&bookmark.name) != room.name().as_ref()
                    {
                        update_bookmarks_futures.push(
                            async move {
                                self.save_bookmark_for_room(&room).await;
                            }
                            .prose_boxed(),
                        );
                    }
                }
                Ok(_) => (),
                Err(error)
                    if (error.is_gone_err() || error.is_registration_required_err())
                        && !bookmark.sidebar_state.is_in_sidebar() =>
                {
                    // If a room that is hidden from the sidebar is gone or we're not
                    // a member (anymore), we'll delete the corresponding bookmark.
                    info!("Deleting bookmark for hidden gone room {room_id}…");
                    update_bookmarks_futures.push(
                        async move {
                            self.connected_rooms_repo.delete(&room_id);
                            self.delete_bookmark(&room_id).await;
                        }
                        .prose_boxed(),
                    );
                }
                Err(error) => {
                    error!(
                        "Failed to join room '{}' from bookmark. Reason: {}. is_subscription_required_err? {}",
                        room_id,
                        error.to_string(),
                        error.is_registration_required_err()
                    )
                }
            }
        }

        // Now on to bookkeeping…
        join_all(update_bookmarks_futures).await;

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
    ) -> Result<RoomId> {
        let result = self
            .rooms_domain_service
            .create_or_join_room(request, RoomSidebarState::InSidebar)
            .await;

        let room = match result {
            Ok(room) => room,
            Err(RoomError::RoomIsAlreadyConnected(room_id)) => {
                let Some(room) = self.connected_rooms_repo.get(&room_id) else {
                    return Err(format_err!("Failed to join room. Please try again."));
                };
                if room.sidebar_state().is_in_sidebar() {
                    return Ok(room_id);
                }
                room.set_sidebar_state(RoomSidebarState::InSidebar);
                room
            }
            Err(error) => return Err(error.into()),
        };

        // If the room already existed and was silently returned by the RoomsDomainService, make
        // sure that it actually is configured to show up in the sidebar…
        if !room.sidebar_state().is_in_sidebar() {
            room.set_sidebar_state(RoomSidebarState::InSidebar);
        }

        self.save_bookmark_for_room(&room).await;

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(room.room_id.clone())
    }

    /// Ensures a sidebar item exists for an active direct message or group conversation.
    ///
    /// If a message is received from a direct message or group that is not currently represented
    /// in the sidebar, this method will insert an item into the sidebar and update the
    /// corresponding bookmark.
    ///
    /// Dispatches a `ClientEvent::SidebarChanged` event after processing.
    async fn handle_received_message(&self, sender: &UserEndpointId) -> Result<()> {
        let room_id = sender.to_room_id();

        let room = match sender {
            // We do not need to create or join a room here since we couldn't have received
            // a message from a room we're not connected to. Also we always stay connected to rooms
            // for groups no matter if they are in the sidebar or not.
            UserEndpointId::Occupant(_) => self.try_get_room(&room_id)?,

            // If the message is from a user outside of a room we create a room if we don't
            // have one yet.
            UserEndpointId::User(_) | UserEndpointId::UserResource(_) => 'room: {
                if let Some(room) = self.connected_rooms_repo.get(&room_id) {
                    break 'room room;
                };
                self.rooms_domain_service
                    .create_or_join_room(
                        CreateOrEnterRoomRequest::JoinDirectMessage {
                            participant: UserId::from(room_id.into_inner()),
                        },
                        RoomSidebarState::NotInSidebar,
                    )
                    .await?
            }
        };

        room.increment_unread_count();

        match room.r#type {
            RoomType::DirectMessage => (),
            RoomType::Group => (),
            _ => {
                self.client_event_dispatcher
                    .dispatch_event(ClientEvent::SidebarChanged);
                return Ok(());
            }
        };

        if !room.sidebar_state().is_in_sidebar() {
            room.set_sidebar_state(RoomSidebarState::InSidebar);
            self.save_bookmark_for_room(&room).await;
        }

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }

    async fn destroy_room(&self, room_id: &RoomId) -> Result<()> {
        if self.connected_rooms_repo.get(room_id).is_none() {
            return Ok(());
        }

        match self
            .room_management_service
            .destroy_room(room_id, None)
            .await
        {
            Ok(_) => (),
            // The room is gone but we are still somehow connected to it. Maybe an outdated
            // bookmark? In this case let's proceed with deleting the room and the bookmark.
            Err(err) if err.is_gone_err() => (),
            Err(err) => {
                return Err(err.into());
            }
        }

        self.connected_rooms_repo.delete(room_id);
        self.delete_bookmark(room_id).await;

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }

    /// Renames the sidebar item identified by `room_id` to `name`.
    ///
    /// If the item is not in the list of sidebar items no action is performed, otherwise:
    ///   - The corresponding room will be renamed.
    ///   - The corresponding bookmark will be renamed.
    ///   - `ClientEvent::SidebarChanged` will be dispatched after processing.
    async fn rename_item(&self, room_id: &RoomId, name: &str) -> Result<()> {
        let room = self.try_get_room(room_id).context("Cannot rename room.")?;

        let current_name = room.name();

        // Nothing changed.
        if current_name.as_deref().unwrap_or_default().to_lowercase() == name.to_lowercase() {
            return Ok(());
        }
        room.set_name(Some(name.to_string()));

        // Rename the room and reset the name in case the operation fails.
        match self.rooms_domain_service.rename_room(room_id, name).await {
            Ok(_) => (),
            Err(err) => {
                room.set_name(current_name);
                return Err(err.into());
            }
        }

        self.save_bookmark_for_room(&room).await;

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }

    /// Toggles the `is_favorite` flag for the sidebar item identified by `room_id`.
    ///
    /// If the item is not in the list of sidebar items no action is performed, otherwise:
    ///   - The corresponding bookmark will be updated to reflect the new status of `is_favorite`.
    ///   - `ClientEvent::SidebarChanged` will be dispatched after processing.
    async fn toggle_item_is_favorite(&self, room_id: &RoomId) -> Result<()> {
        let room = self
            .try_get_room(room_id)
            .with_context(|| format!("Cannot toggle favorite status of room '{room_id}'"))?;

        room.set_sidebar_state(match room.sidebar_state() {
            RoomSidebarState::NotInSidebar => return Ok(()),
            RoomSidebarState::InSidebar => RoomSidebarState::Favorite,
            RoomSidebarState::Favorite => RoomSidebarState::InSidebar,
        });

        self.save_bookmark_for_room(&room).await;

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }

    /// Reconfigures the sidebar item identified by `room_id` according to `spec`.
    ///
    /// If the item is not in the list of sidebar items no action is performed, otherwise:
    ///   - The corresponding room will be reconfigured.
    ///   - The corresponding bookmark's type will be updated.
    ///   - `ClientEvent::SidebarChanged` will be dispatched after processing.
    #[tracing::instrument(skip(self))]
    async fn reconfigure_item_with_spec(
        &self,
        room_id: &RoomId,
        spec: RoomSpec,
        new_name: &str,
    ) -> Result<()> {
        info!("Reconfiguring room {} to type {}…", room_id, spec);

        let room = self
            .rooms_domain_service
            .reconfigure_room_with_spec(room_id, spec, new_name)
            .await?;

        info!(
            "Reconfiguration of room {} finished. Room Jid is now {}",
            room_id, room.room_id
        );

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        // The returned room has a new JID, which implies that the old room has been deleted…
        if room_id != &room.room_id {
            self.delete_bookmark(room_id).await;
        }

        self.save_bookmark_for_room(&room).await;
        Ok(())
    }

    /// Removes multiple sidebar items associated with the provided `room_ids`.
    ///
    /// - Disconnects channels and updates the repository state for each provided JID.
    /// - Groups and Private Channels have their bookmarks updated to reflect they are not in
    ///   the sidebar.
    /// - DirectMessages and Public Channels are deleted from bookmarks, as they do not require
    ///   persistent connections and can be rediscovered.
    /// - Triggers a `ClientEvent::SidebarChanged` event after processing to notify of the
    ///   sidebar update.
    async fn remove_items(&self, room_ids: &[&RoomId]) -> Result<()> {
        for &room_id in room_ids {
            let Some(room) = self.connected_rooms_repo.get(room_id) else {
                return Ok(());
            };

            self.disconnect_and_delete_room(&room).await;

            match room.r#type {
                // For Groups and Private Channels we do not really delete the bookmarks. The reason
                // is that Groups should always be connected so that our user can receive messages from
                // them, while we keep references to the Private channels because we'd otherwise loose
                // track of them since the MUC service at this time only let's us discover
                // public channels.
                RoomType::Group | RoomType::PrivateChannel => {
                    room.set_sidebar_state(RoomSidebarState::NotInSidebar);
                    self.save_bookmark_for_room(&room).await;
                }
                RoomType::DirectMessage | RoomType::PublicChannel | RoomType::Generic => {
                    self.delete_bookmark(room_id).await;
                }
                RoomType::Unknown => (),
            }
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
    async fn handle_removed_items(&self, room_ids: &[RoomId]) -> Result<()> {
        for id in room_ids {
            let Some(room) = self.connected_rooms_repo.get(id) else {
                continue;
            };
            self.disconnect_and_delete_room(&room).await;
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
        // No need to delete the bookmarks here since that is the raison d'etre for this method.
        // We'll only need to delete the connected rooms.
        for room in self.connected_rooms_repo.delete_all() {
            if room.r#type == RoomType::DirectMessage {
                continue;
            }
            let full_jid = room.user_full_jid();
            self.room_management_service.exit_room(&full_jid).await?;
        }

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }

    /// Handles a destroyed room.
    ///
    /// - Removes the connected room.
    /// - Joins `alternate_room` if set (see `insert_item_by_creating_or_joining_room`).
    /// - Dispatches a `ClientEvent::SidebarChanged` event after processing.
    async fn handle_destroyed_room(
        &self,
        room_id: &RoomId,
        alternate_room: Option<RoomId>,
    ) -> Result<()> {
        let Some(alternate_room) = alternate_room else {
            let Some(room) = self.connected_rooms_repo.get(room_id) else {
                return Ok(());
            };

            room.set_state(RoomState::Disconnected {
                error: Some("This room has been destroyed.".to_string()),
                can_retry: false,
            });

            self.client_event_dispatcher
                .dispatch_event(ClientEvent::SidebarChanged);

            return Ok(());
        };

        // Remove the destroyed room…
        let Some(room) = self.connected_rooms_repo.delete(room_id) else {
            return Ok(());
        };

        // We're already connected to the alternate room.
        if self.connected_rooms_repo.get(&alternate_room).is_some() {
            self.delete_bookmark(&alternate_room).await;
            return Ok(());
        }

        // …and insert a pending room with the same name instead…
        _ = self.connected_rooms_repo.set(Room::pending(
            &Bookmark {
                name: room.name().unwrap_or_else(|| room.room_id.to_string()),
                jid: alternate_room.clone(),
                r#type: room.r#type.into(),
                sidebar_state: room.sidebar_state(),
            },
            &build_nickname(&self.ctx.connected_id()?.to_user_id()),
        ));

        // Let the UI update the sidebar…
        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        join_all([
            async {
                self.delete_bookmark(room_id).await;
            }
            .prose_boxed(),
            async move {
                let result = self
                    .rooms_domain_service
                    .create_or_join_room(
                        CreateOrEnterRoomRequest::JoinRoom {
                            room_id: alternate_room.clone(),
                            password: None,
                            behavior: JoinRoomBehavior::system_initiated(),
                        },
                        room.sidebar_state(),
                    )
                    .await;

                match result {
                    Ok(room) => self.save_bookmark_for_room(&room).await,
                    Err(error) => error!(
                        "Failed to join alternate room {alternate_room}. Reason: {}",
                        error.to_string()
                    ),
                }
            }
            .prose_boxed(),
        ])
        .await;

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }

    /// Handles removal from a room.
    ///
    /// If the removal is temporary:
    /// - Deletes the connected room.
    /// - Sets an error on the corresponding sidebar item.
    /// - Dispatches a `ClientEvent::SidebarChanged` event after processing.
    ///
    /// If the removal is permanent, follows the procedure described in `handle_destroyed_room`.
    async fn handle_removal_from_room(&self, room_id: &RoomId, is_permanent: bool) -> Result<()> {
        let Some(room) = self.connected_rooms_repo.get(room_id) else {
            return Ok(());
        };

        room.set_state(RoomState::Disconnected {
            error: Some(format!(
                "You've been {} removed from this room.",
                if is_permanent {
                    "permanently"
                } else {
                    "temporarily"
                }
            )),
            can_retry: !is_permanent,
        });

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }

    /// Handles a changed room configuration.
    ///
    /// - Reloads the configuration and adjusts the connected room accordingly.
    /// - Replaces the connected room if the type of room changed.
    /// - Updates the sidebar & associated bookmark to reflect the updated configuration.
    /// - Dispatches a `ClientEvent::SidebarChanged` event after processing.
    async fn handle_changed_room_config(&self, room_id: &RoomId) -> Result<()> {
        let Some(room) = self.connected_rooms_repo.get(room_id) else {
            return Ok(());
        };

        // Ignore connecting rooms, since the message might be generated by us creating the room…
        if room.is_connecting() {
            return Ok(());
        }

        let former_name = room.name();
        let former_type = room.r#type;

        let room = self
            .rooms_domain_service
            .reevaluate_room_spec(room_id)
            .await?;

        let new_name = room.name();
        let new_type = room.r#type;

        if new_name == former_name && new_type == former_type {
            info!("No changes required for bookmark {}.", room_id);
            return Ok(());
        }

        self.save_bookmark_for_room(&room).await;

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }

    /// Removes all connected rooms and sidebar items.
    ///
    /// Call this method after logging out.
    async fn clear_cache(&self) -> Result<()> {
        self.connected_rooms_repo.delete_all();
        Ok(())
    }
}

impl SidebarDomainService {
    /// Disconnects from a room associated with a removed sidebar item if necessary.
    ///
    /// - DirectMessages are not disconnected as they are not MUC rooms and do not require
    ///   disconnection.
    /// - Groups, being MUC rooms, should remain connected to ensure users always receive messages
    ///   and are not disconnected.
    /// - Private and Public Channels are disconnected when they are removed from the sidebar.
    ///
    /// If a room is disconnected, it is removed from the `ConnectedRoomsRepository`.
    async fn disconnect_and_delete_room(&self, room: &Room) {
        match room.r#type {
            // DirectMessages do not need to be connected as they are not MUC rooms
            RoomType::DirectMessage => {
                self.connected_rooms_repo.delete(&room.room_id);
            }
            // Groups will always be connected so that they behave like DirectMessages insofar that
            // our user should always receive messages from them.
            RoomType::Group => room.set_sidebar_state(RoomSidebarState::NotInSidebar),
            // Private and Public Channels actually will be disconnected.
            RoomType::PrivateChannel | RoomType::PublicChannel | RoomType::Generic => {
                let full_jid = room.user_full_jid();
                self.connected_rooms_repo.delete(&room.room_id);

                match self.room_management_service.exit_room(&full_jid).await {
                    Ok(_) => (),
                    Err(error) => error!("Failed to exit room. Reason: {}", error.to_string()),
                }
            }
            RoomType::Unknown => (),
        }
    }

    /// Attempts to join a room based on the given `bookmark`. Returns `None` if the room doesn't
    /// need to be connected to, i.e. a Public Channel that is not in the sidebar.
    ///
    /// - DirectMessage rooms are not joined as they are not MUC rooms.
    /// - Channels are only joined if they appear in the sidebar (`Bookmark::in_sidebar`).
    /// - Groups are always joined, regardless of sidebar status, to ensure message receipt.
    ///
    /// If a room is joined, it is added to the `ConnectedRoomsRepository`.
    async fn join_room_identified_by_bookmark_if_needed(
        &self,
        bookmark: &Bookmark,
        behavior: JoinRoomBehavior,
    ) -> Result<Option<Room>, RoomError> {
        let room = match bookmark.r#type {
            BookmarkType::DirectMessage if !bookmark.sidebar_state.is_in_sidebar() => None,

            // For channels, we're only participating in them if they're in the sidebar.
            BookmarkType::PublicChannel | BookmarkType::PrivateChannel | BookmarkType::Generic
                if !bookmark.sidebar_state.is_in_sidebar() =>
            {
                None
            }

            // Since direct messages are not MUC rooms we don't need to connect to them. But we'll
            // insert the placeholder room instead.
            BookmarkType::DirectMessage => Some(
                self.rooms_domain_service
                    .create_or_join_room(
                        CreateOrEnterRoomRequest::JoinDirectMessage {
                            participant: UserId::from(bookmark.jid.clone().into_inner()),
                        },
                        bookmark.sidebar_state,
                    )
                    .await?,
            ),

            // While our user can remove a Group from their sidebar they should always receive
            // messages from it. In these cases the Group will automatically reappear in the
            // sidebar. We want our users to think about Groups as if they were a
            // Direct Message.
            BookmarkType::Group
            | BookmarkType::PublicChannel
            | BookmarkType::PrivateChannel
            | BookmarkType::Generic => Some(
                self.rooms_domain_service
                    .create_or_join_room(
                        CreateOrEnterRoomRequest::JoinRoom {
                            room_id: bookmark.jid.clone(),
                            password: None,
                            behavior,
                        },
                        bookmark.sidebar_state,
                    )
                    .await?,
            ),
        };

        Ok(room)
    }
}

impl SidebarDomainService {
    /// Saves a bookmark for `room`. Errors will be logged but otherwise ignored.
    async fn save_bookmark_for_room(&self, room: &Room) {
        info!("Saving bookmark for room {}…", room.room_id);

        let bookmark = match Bookmark::try_from(room) {
            Ok(bookmark) => bookmark,
            Err(err) => {
                error!("{}", err.to_string());
                return;
            }
        };

        if let Err(err) = self.bookmarks_service.save_bookmark(&bookmark).await {
            error!("Failed to save bookmark. Reason: {}", err.to_string());
        }
    }

    /// Deletes the bookmark for `room_id`. Errors will be logged but otherwise ignored.
    async fn delete_bookmark(&self, room_id: &RoomId) {
        info!("Deleting bookmark for room {}…", room_id);

        if let Err(err) = self.bookmarks_service.delete_bookmark(&room_id).await {
            error!("Failed to delete bookmark. Reason: {}", err.to_string());
        }
    }

    fn try_get_room(&self, room_id: &RoomId) -> Result<Room> {
        let Some(room) = self.connected_rooms_repo.get(room_id) else {
            bail!("No room with id '{room_id}'")
        };
        Ok(room)
    }
}

impl TryFrom<&Room> for Bookmark {
    type Error = anyhow::Error;

    fn try_from(value: &Room) -> Result<Self> {
        let bookmark_type = match value.r#type {
            RoomType::Unknown => {
                return Err(format_err!("Cannot create bookmark for a pending room"))
            }
            RoomType::DirectMessage => BookmarkType::DirectMessage,
            RoomType::Group => BookmarkType::Group,
            RoomType::PrivateChannel => BookmarkType::PrivateChannel,
            RoomType::PublicChannel => BookmarkType::PublicChannel,
            RoomType::Generic => BookmarkType::Generic,
        };

        Ok(Self {
            name: value.name().unwrap_or_else(|| value.room_id.to_string()),
            jid: value.room_id.clone(),
            r#type: bookmark_type,
            sidebar_state: value.sidebar_state(),
        })
    }
}
