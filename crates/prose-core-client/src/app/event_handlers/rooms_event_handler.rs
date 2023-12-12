// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

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
use crate::domain::rooms::models::RoomInternals;
use crate::domain::rooms::services::CreateOrEnterRoomRequest;
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
    fn get_room(&self, jid: &RoomId) -> Result<Arc<RoomInternals>> {
        self.connected_rooms_repo
            .get(jid)
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
                        participants.set_affiliation(&participant_id, &affiliation);
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
                room.participants_mut()
                    .set_availability(&participant_id, &Availability::Unavailable);
                // TODO: If this affects us we should keep the connected room around, but add an error message to it.
                true
            }
            OccupantEventType::PermanentlyRemoved => 'outer: {
                room.participants_mut().remove(&participant_id);

                if event.is_self {
                    self.sidebar_domain_service
                        .handle_removal_from_room(&event.occupant_id.room_id(), true)
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

                let room = self.get_room(&event.room_id)?;
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
                        room_jid: event.room_id,
                        password,
                    })
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_user_status_event(&self, event: UserStatusEvent) -> Result<()> {
        let room = self.get_room(&event.user_id.to_room_id())?;
        let participant_id = event.user_id.to_participant_id();

        match event.r#type {
            UserStatusEventType::AvailabilityChanged {
                availability,
                priority,
            } => {
                room.participants_mut()
                    .set_availability(&participant_id, &availability);

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

                let user_id = id.to_user_id();

                // We won't send an event for our own availability…
                if user_id == self.ctx.connected_id()?.into_user_id() {
                    return Ok(());
                }

                self.client_event_dispatcher
                    .dispatch_event(ClientEvent::ContactChanged { id: user_id });
            }
            UserStatusEventType::ComposeStateChanged { state } => {
                room.participants_mut().set_compose_state(
                    &participant_id,
                    &self.time_provider.now(),
                    state,
                );

                // TODO: Don't send an event when this is about us. Neither in DirectMessages nor in MUC room.

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
