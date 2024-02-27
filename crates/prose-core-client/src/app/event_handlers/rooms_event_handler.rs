// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use tracing::info;

use prose_proc_macros::InjectDependencies;
use prose_xmpp::TimeProvider;

use crate::app::deps::{
    DynAppContext, DynClientEventDispatcher, DynConnectedRoomsReadOnlyRepository,
    DynSidebarDomainService, DynTimeProvider, DynUserInfoRepository, DynUserProfileRepository,
};
use crate::app::event_handlers::ServerEventHandler;
use crate::app::event_handlers::{
    OccupantEvent, OccupantEventType, RoomEvent, RoomEventType, ServerEvent, UserStatusEvent,
    UserStatusEventType,
};
use crate::client_event::ClientRoomEventType;
use crate::domain::messaging::models::{MessageLike, MessageLikePayload};
use crate::domain::rooms::models::Room;
use crate::domain::rooms::services::{
    CreateOrEnterRoomRequest, JoinRoomBehavior, JoinRoomFailureBehavior, JoinRoomRedirectBehavior,
};
use crate::domain::shared::models::{ParticipantId, RoomId};
use crate::domain::user_info::models::Presence;
use crate::dtos::Availability;
use crate::ClientEvent;

#[derive(InjectDependencies)]
pub struct RoomsEventHandler {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    connected_rooms_repo: DynConnectedRoomsReadOnlyRepository,
    #[inject]
    sidebar_domain_service: DynSidebarDomainService,
    #[inject]
    client_event_dispatcher: DynClientEventDispatcher,
    #[inject]
    time_provider: DynTimeProvider,
    #[inject]
    user_profile_repo: DynUserProfileRepository,
    #[inject]
    user_info_repo: DynUserInfoRepository,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ServerEventHandler for RoomsEventHandler {
    fn name(&self) -> &'static str {
        "rooms"
    }

    async fn handle_event(&self, event: ServerEvent) -> Result<Option<ServerEvent>> {
        match event {
            ServerEvent::Occupant(event) => {
                self.handle_occupant_event(event).await?;
            }
            ServerEvent::Room(event) => {
                self.handle_room_event(event).await?;
            }
            ServerEvent::UserStatus(event) => self.handle_user_status_event(event).await?,
            _ => return Ok(Some(event)),
        }
        Ok(None)
    }
}

impl RoomsEventHandler {
    fn get_room(&self, jid: &RoomId) -> Result<Room> {
        self.connected_rooms_repo
            .get(jid.as_ref())
            .ok_or(anyhow::format_err!("Could not find room with jid {}", jid))
    }

    async fn handle_occupant_event(&self, event: OccupantEvent) -> Result<()> {
        let room = self.get_room(&event.occupant_id.room_id())?;
        let participant_id = ParticipantId::Occupant(event.occupant_id.clone());

        let participants_changed = match event.r#type {
            OccupantEventType::AffiliationChanged { affiliation } => 'outer: {
                let mut participants_changed = false;

                {
                    let mut participants = room.participants_mut();
                    if participants.get(&participant_id).map(|p| &p.affiliation)
                        != Some(&affiliation)
                    {
                        participants_changed = true;
                        participants.set_affiliation(&participant_id, event.is_self, &affiliation);
                    }
                }

                // Let's see if we knew the real id of the participant already, if not let's
                // look up their name…
                let (Some(real_id), Some(participant)) = (
                    event.real_id,
                    room.participants().get(&participant_id).cloned(),
                ) else {
                    break 'outer participants_changed;
                };

                if participant.real_id.is_some() {
                    // Real id was known already…
                    break 'outer participants_changed;
                }

                let name = self.user_profile_repo.get_display_name(&real_id).await?;
                room.participants_mut().set_ids_and_name(
                    &participant_id,
                    Some(&real_id),
                    event.anon_occupant_id.as_ref(),
                    name.as_deref(),
                );

                true
            }
            OccupantEventType::DisconnectedByServer => {
                room.participants_mut().set_availability(
                    &participant_id,
                    event.is_self,
                    &Availability::Unavailable,
                );

                if event.is_self {
                    self.sidebar_domain_service
                        .handle_removal_from_room(&event.occupant_id.muc_id(), false)
                        .await?;
                }

                true
            }
            OccupantEventType::PermanentlyRemoved => 'outer: {
                room.participants_mut().remove(&participant_id);

                if event.is_self {
                    self.sidebar_domain_service
                        .handle_removal_from_room(&event.occupant_id.muc_id(), true)
                        .await?;
                    // A SidebarChanged event will be sent instead
                    break 'outer false;
                }

                true
            }
        };

        if participants_changed {
            self.client_event_dispatcher
                .dispatch_room_event(room, ClientRoomEventType::ParticipantsChanged);
        }

        Ok(())
    }

    async fn handle_room_event(&self, event: RoomEvent) -> Result<()> {
        match event.r#type {
            RoomEventType::Destroyed { replacement } => {
                info!(
                    "Room {} was destroyed. Alternative is {:?}",
                    event.room_id, replacement
                );
                self.sidebar_domain_service
                    .handle_destroyed_room(&event.room_id, replacement)
                    .await?;
            }
            RoomEventType::RoomConfigChanged => {
                info!("Config changed for room {}.", event.room_id);
                self.sidebar_domain_service
                    .handle_changed_room_config(&event.room_id)
                    .await?;
            }
            RoomEventType::RoomTopicChanged { new_topic } => {
                info!(
                    "Updating topic of room {} to '{:?}'",
                    event.room_id, new_topic
                );

                let room = self.get_room(&RoomId::Muc(event.room_id))?;
                if room.topic() != new_topic {
                    room.set_topic(new_topic);
                    self.client_event_dispatcher
                        .dispatch_room_event(room, ClientRoomEventType::AttributesChanged)
                }
            }
            RoomEventType::ReceivedInvitation { sender, password } => {
                info!(
                    "Joining room {} after receiving invitation from {sender}…",
                    event.room_id
                );
                self.sidebar_domain_service
                    .insert_item_by_creating_or_joining_room(CreateOrEnterRoomRequest::JoinRoom {
                        room_id: event.room_id,
                        password,
                        behavior: JoinRoomBehavior {
                            on_redirect: JoinRoomRedirectBehavior::FollowIfGone,
                            on_failure: JoinRoomFailureBehavior::RetainOnError,
                        },
                    })
                    .await?;
            }
            RoomEventType::UserAdded {
                user_id,
                affiliation,
                reason,
            } => {
                info!(
                    "User {user_id} was added to room {} via invitation. Reason: {}",
                    event.room_id,
                    reason.as_deref().unwrap_or("<no reason>")
                );

                let room = self.get_room(&RoomId::Muc(event.room_id))?;

                let name = self.user_profile_repo.get_display_name(&user_id).await?;
                room.participants_mut()
                    .add_user(&user_id, false, &affiliation, name.as_deref());

                self.client_event_dispatcher
                    .dispatch_room_event(room, ClientRoomEventType::ParticipantsChanged);
            }
        }

        Ok(())
    }

    async fn handle_user_status_event(&self, event: UserStatusEvent) -> Result<()> {
        let is_self_event =
            event.user_id.to_user_id() == Some(self.ctx.connected_id()?.into_user_id());

        match event.r#type {
            UserStatusEventType::AvailabilityChanged {
                availability,
                priority,
            } => {
                let mut room_changed = false;

                // If we have a room, update it…
                if let Ok(room) = self.get_room(&event.user_id.to_room_id()) {
                    let participant_id = event.user_id.to_participant_id();
                    room.participants_mut().set_availability(
                        &participant_id,
                        is_self_event,
                        &availability,
                    );

                    if room.sidebar_state().is_in_sidebar() {
                        if event.user_id.is_occupant_id() {
                            // The participant list should be reloaded in the UI to reflect
                            // the new availability…
                            self.client_event_dispatcher.dispatch_room_event(
                                room,
                                ClientRoomEventType::ParticipantsChanged,
                            );
                        }

                        // If this is a DM room, a SidebarChanged event will be fired down the
                        // line, since the UI displays an availability indicator.
                        room_changed = true;
                    }
                };

                // if we do not have a room and the event is from a contact, we'll still want
                // to update our repo…
                let Some(id) = event.user_id.to_user_or_resource_id() else {
                    return Ok(());
                };

                self.user_info_repo
                    .set_user_presence(
                        &id,
                        &Presence {
                            priority,
                            availability,
                            status: None,
                        },
                    )
                    .await?;

                // We won't send an event for our own availability…
                if is_self_event {
                    return Ok(());
                }

                self.client_event_dispatcher
                    .dispatch_event(ClientEvent::ContactChanged {
                        ids: vec![id.to_user_id()],
                    });

                if room_changed {
                    self.client_event_dispatcher
                        .dispatch_event(ClientEvent::SidebarChanged)
                }
            }
            UserStatusEventType::ComposeStateChanged { state } => {
                let Ok(room) = self.get_room(&event.user_id.to_room_id()) else {
                    return Ok(());
                };
                let participant_id = event.user_id.to_participant_id();

                room.participants_mut().set_compose_state(
                    &participant_id,
                    &self.time_provider.now(),
                    state,
                );

                // We won't send an event for our own compose state…
                if is_self_event {
                    return Ok(());
                }

                self.client_event_dispatcher
                    .dispatch_room_event(room, ClientRoomEventType::ComposingUsersChanged);
            }
        }

        Ok(())
    }
}

impl From<&MessageLike> for ClientRoomEventType {
    fn from(message: &MessageLike) -> Self {
        if let Some(ref target) = message.target {
            if message.payload == MessageLikePayload::Retraction {
                Self::MessagesDeleted {
                    message_ids: vec![target.as_ref().into()],
                }
            } else {
                Self::MessagesUpdated {
                    message_ids: vec![target.as_ref().into()],
                }
            }
        } else {
            Self::MessagesAppended {
                message_ids: vec![message.id.id().as_ref().into()],
            }
        }
    }
}
