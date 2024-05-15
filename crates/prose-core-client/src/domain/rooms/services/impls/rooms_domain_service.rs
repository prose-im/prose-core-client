// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::future::Future;
use std::iter;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use jid::{BareJid, NodePart};
use tracing::{debug, error, info, warn};

use prose_proc_macros::DependenciesStruct;
use prose_xmpp::{IDProvider, RequestError};

use crate::app::deps::{
    DynAccountSettingsRepository, DynAppContext, DynClientEventDispatcher,
    DynConnectedRoomsRepository, DynIDProvider, DynMessageArchiveDomainService,
    DynMessageMigrationDomainService, DynRoomAttributesService, DynRoomManagementService,
    DynRoomParticipationService, DynUserInfoRepository, DynUserProfileRepository,
};
use crate::domain::general::models::Capabilities;
use crate::domain::rooms::models::{
    RegisteredMember, Room, RoomAffiliation, RoomError, RoomFeatures, RoomInfo, RoomSessionInfo,
    RoomSessionMember, RoomSidebarState, RoomSpec,
};
use crate::domain::rooms::services::rooms_domain_service::{
    CreateRoomBehavior, JoinRoomFailureBehavior, JoinRoomRedirectBehavior,
};
use crate::domain::rooms::services::{CreateOrEnterRoomRequest, JoinRoomBehavior};
use crate::domain::shared::models::{MucId, RoomId, RoomType, UserId};
use crate::dtos::{Availability, RoomState};
use crate::util::StringExt;
use crate::ClientRoomEventType;

use super::super::{CreateRoomType, RoomsDomainService as RoomsDomainServiceTrait};
use super::{build_nickname, ParticipantsVecExt};

const CHANNEL_PREFIX: &str = "org.prose.channel";

#[derive(DependenciesStruct)]
pub struct RoomsDomainService {
    account_settings_repo: DynAccountSettingsRepository,
    client_event_dispatcher: DynClientEventDispatcher,
    connected_rooms_repo: DynConnectedRoomsRepository,
    ctx: DynAppContext,
    id_provider: DynIDProvider,
    message_migration_domain_service: DynMessageMigrationDomainService,
    room_attributes_service: DynRoomAttributesService,
    room_management_service: DynRoomManagementService,
    room_participation_service: DynRoomParticipationService,
    user_info_repo: DynUserInfoRepository,
    user_profile_repo: DynUserProfileRepository,
    message_archive_domain_service: DynMessageArchiveDomainService,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl RoomsDomainServiceTrait for RoomsDomainService {
    async fn create_or_join_room(
        &self,
        request: CreateOrEnterRoomRequest,
        sidebar_state: RoomSidebarState,
    ) -> Result<Room, RoomError> {
        let room = match request {
            CreateOrEnterRoomRequest::Create {
                service,
                room_type,
                behavior,
            } => {
                self.create_room(&service, room_type, sidebar_state, behavior)
                    .await
            }
            CreateOrEnterRoomRequest::JoinRoom {
                room_id: room_jid,
                password,
                behavior,
            } => {
                self.join_room(&room_jid, password.as_deref(), sidebar_state, behavior)
                    .await
            }
            CreateOrEnterRoomRequest::JoinDirectMessage { participant } => {
                self.join_direct_message(&participant, sidebar_state).await
            }
        }?;

        self.message_archive_domain_service
            .catchup_room(&room)
            .await?;

        Ok(room)
    }

    /// Renames the room identified by `room_jid` to `name`.
    ///
    /// - If the room is not connected no action is performed.
    /// - Panics if the Room is not of type `RoomType::PublicChannel`, `RoomType::PrivateChannel`
    ///   or `RoomType::Generic`.
    /// - Fails with `RoomError::PublicChannelNameConflict` if the room is of type
    ///   `RoomType::PublicChannel` and `name` is already used by another public channel.
    async fn rename_room(&self, room_id: &MucId, name: &str) -> Result<(), RoomError> {
        let Some(room) = self.connected_rooms_repo.get(room_id.as_ref()) else {
            return Err(RoomError::RoomNotFound);
        };

        match room.r#type {
            // We do not allow renaming Direct Messages or Groups since those names should always
            // represent the list of participants.
            RoomType::Unknown | RoomType::DirectMessage | RoomType::Group => {
                panic!("Unsupported action")
            }
            RoomType::PublicChannel => {
                // Ensure that the new name doesn't exist already.
                if !self.is_public_channel_name_unique(name).await? {
                    return Err(RoomError::PublicChannelNameConflict);
                }
            }
            RoomType::PrivateChannel | RoomType::Generic => (),
        }

        self.room_attributes_service
            .set_name(room_id, name.as_ref())
            .await?;
        room.set_name(Some(name.to_string()));

        self.client_event_dispatcher
            .dispatch_room_event(room, ClientRoomEventType::AttributesChanged);

        Ok(())
    }

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
    ) -> Result<Room, RoomError> {
        let Some(room) = self.connected_rooms_repo.get(room_id.as_ref()) else {
            return Err(RoomError::RoomNotFound);
        };

        match (&room.r#type, spec.room_type()) {
            (RoomType::Group, RoomType::PrivateChannel) => {
                // Remove room first so that we don't run into problems with reentrancy…
                self.connected_rooms_repo.delete(room_id.as_ref());

                let service = BareJid::from_parts(None, &room_id.domain());

                // Create new room
                debug!("Creating new room {}…", new_name);
                let new_room = match self
                    .create_room(
                        &service,
                        CreateRoomType::PrivateChannel {
                            name: new_name.to_string(),
                        },
                        room.sidebar_state(),
                        CreateRoomBehavior::FailIfGone,
                    )
                    .await
                {
                    Ok(room) => room,
                    Err(err) => {
                        // Something went wrong, let's put the room back…
                        _ = self.connected_rooms_repo.set(room);
                        return Err(err);
                    }
                };

                let new_room_id = new_room
                    .room_id
                    .muc_id()
                    .ok_or(anyhow!("Expected new room to be a MUC room."))?;

                // Migrate messages to new room
                debug!("Copying messages to new room {}…", new_name);
                match self
                    .message_migration_domain_service
                    .copy_all_messages_from_room(&room.room_id, &new_room.room_id)
                    .await
                {
                    Ok(_) => (),
                    Err(err) => {
                        // If that failed, let's put the initial room back and delete the new room?!
                        _ = self.connected_rooms_repo.set(room);
                        _ = self
                            .room_management_service
                            .destroy_room(new_room_id, None)
                            .await;
                        return Err(err.into());
                    }
                }

                let current_user = self.ctx.connected_id()?.to_user_id();
                let member_ids = room
                    .participants()
                    .values()
                    .filter_map(|p| {
                        if p.affiliation >= RoomAffiliation::Member {
                            return p.real_id.clone();
                        }
                        None
                    })
                    .collect::<Vec<_>>();

                // Now grant the members of the original group access to the new channel…
                debug!("Granting membership to members of new room {}…", new_name);
                for member in member_ids {
                    // Our user is already admin, no need to set them as a member…
                    if member == current_user {
                        continue;
                    }

                    match self
                        .room_participation_service
                        .grant_membership(new_room_id, &member)
                        .await
                    {
                        Ok(_) => (),
                        Err(err) => {
                            error!(
                                "Could not grant membership for new private channel {} to {}. Reason: {}",
                                new_room.room_id, member, err.to_string()
                            );
                        }
                    }
                }

                // And finally destroy the original room. Since we pass in the JID to the new room
                // we do not need to send invites to the members of the original group.
                debug!("Destroying old room {}…", room.room_id);
                match self
                    .room_management_service
                    .destroy_room(room_id, new_room.room_id.muc_id().cloned())
                    .await
                {
                    Ok(_) => (),
                    Err(err) => {
                        // If that failed, no reason to stop here. Let's just log the error…
                        warn!("Failed to delete the initial group after trying to convert it to a Private Channel. Reason: {}", err.to_string());
                    }
                }

                Ok(new_room)
            }
            (RoomType::PrivateChannel, RoomType::PublicChannel) => {
                // Ensure that the new name doesn't exist already.
                if !self.is_public_channel_name_unique(new_name).await? {
                    return Err(RoomError::PublicChannelNameConflict);
                }

                self.room_management_service
                    .reconfigure_room(room_id, spec, new_name)
                    .await?;

                let Some(room) = self.connected_rooms_repo.update(room_id.as_ref(), {
                    Box::new(|room| room.by_changing_type(RoomType::PublicChannel))
                }) else {
                    return Err(RequestError::Generic {
                        msg: "Room was modified during reconfiguration".to_string(),
                    }
                    .into());
                };

                Ok(room)
            }
            (RoomType::PublicChannel, RoomType::PrivateChannel) => {
                self.room_management_service
                    .reconfigure_room(room_id, spec, new_name)
                    .await?;

                // TODO: Make public channels also members-only so that the member list translates to the private channel

                let Some(room) = self.connected_rooms_repo.update(room_id.as_ref(), {
                    Box::new(|room| room.by_changing_type(RoomType::PrivateChannel))
                }) else {
                    return Err(RequestError::Generic {
                        msg: "Room was modified during reconfiguration".to_string(),
                    }
                    .into());
                };

                Ok(room)
            }
            (RoomType::Group, _)
            | (RoomType::PrivateChannel, _)
            | (RoomType::PublicChannel, _)
            | (RoomType::DirectMessage, _)
            | (RoomType::Unknown, _)
            | (RoomType::Generic, _) => {
                panic!(
                    "Cannot convert room of type {} to type {}.",
                    room.r#type,
                    spec.room_type()
                );
            }
        }
    }

    /// Loads the configuration for `room_id` and updates the corresponding `RoomInternals`
    /// accordingly. Call this method after the room configuration changed.
    /// Returns `RoomError::RoomNotFound` if no room with `room_id` exists.
    async fn reevaluate_room_spec(&self, room_id: &MucId) -> Result<Room, RoomError> {
        let Some(room) = self.connected_rooms_repo.get(room_id.as_ref()) else {
            return Err(RoomError::RoomNotFound);
        };

        let config = self
            .room_management_service
            .load_room_config(room_id)
            .await?;

        room.set_name(config.room_name);
        room.set_description(config.room_description);

        if room.r#type == config.room_type {
            info!("Room type remained for {}.", room_id);
            return Ok(room);
        }

        info!(
            "Room type changed from {} to {} for {}.",
            room.r#type, config.room_type, room_id
        );

        self.connected_rooms_repo
            .update(
                room_id.as_ref(),
                Box::new(move |room| room.by_changing_type(config.room_type)),
            )
            .ok_or(RoomError::RoomWasModified)
    }
}

impl RoomsDomainService {
    #[tracing::instrument(name = "Join room", skip(self, room_id, password), fields(room_id = %room_id))]
    async fn join_room(
        &self,
        room_id: &MucId,
        password: Option<&str>,
        sidebar_state: RoomSidebarState,
        behavior: JoinRoomBehavior,
    ) -> Result<Room, RoomError> {
        let remove_or_retain_room_on_error =
            |room: Room, error: &RoomError| match behavior.on_failure {
                JoinRoomFailureBehavior::RemoveOnError => {
                    self.connected_rooms_repo.delete(room_id.as_ref());
                }
                JoinRoomFailureBehavior::RetainOnError => room.set_state(RoomState::Disconnected {
                    error: Some(error.to_string()),
                    can_retry: true,
                }),
            };

        let nickname = build_nickname(&self.ctx.connected_id()?.to_user_id());
        let mut room_id = room_id.clone();
        let availability = self
            .account_settings_repo
            .get(&self.ctx.connected_id()?.to_user_id())
            .await?
            .availability;
        let capabilities = &self.ctx.capabilities;

        let info = 'info: loop {
            // Insert pending room so that we don't miss any stanzas for this room while we're
            // connecting to it…
            let room = self.insert_connecting_room(&room_id, &nickname, sidebar_state)?;

            let full_room_jid = room_id.occupant_id_with_nickname(&nickname)?;

            match self
                .room_management_service
                .join_room(&full_room_jid, password, capabilities, availability)
                .await
            {
                Ok(info) => break 'info info,
                Err(error) => {
                    let Some(gone_error) = error.gone_err() else {
                        remove_or_retain_room_on_error(room, &error);
                        return Err(error);
                    };

                    match (behavior.on_redirect, gone_error.new_location) {
                        (JoinRoomRedirectBehavior::FollowIfGone, Some(new_location)) => {
                            self.connected_rooms_repo.delete(room_id.as_ref());
                            room_id = new_location;
                            continue;
                        }
                        (JoinRoomRedirectBehavior::FollowIfGone, None)
                        | (JoinRoomRedirectBehavior::FailIfGone, _) => {
                            remove_or_retain_room_on_error(room, &error);
                            return Err(error);
                        }
                    }
                }
            };
        };

        self.finalize_pending_room(info).await
    }

    async fn join_direct_message(
        &self,
        participant: &UserId,
        sidebar_state: RoomSidebarState,
    ) -> Result<Room, RoomError> {
        match self.connected_rooms_repo.get(participant.as_ref()) {
            Some(room) if room.state() == RoomState::Pending => (),
            None => (),
            Some(room) => return Ok(room),
        }

        let contact_name = self.user_profile_repo.get_display_name(participant).await;
        let user_info = self.user_info_repo.get_user_info(participant).await;

        // Let's ignore potential errors here since the information we're gathering is optional…
        let contact_name = contact_name
            .unwrap_or_default()
            .unwrap_or_else(|| participant.formatted_username());
        let user_info = user_info.unwrap_or_default().unwrap_or_default();

        let room = Room::for_direct_message(
            &participant,
            &contact_name,
            user_info.availability,
            sidebar_state,
            RoomFeatures {
                mam_version: self.ctx.mam_version(),
            },
        );

        self.connected_rooms_repo.set_or_replace(room.clone());

        Ok(room)
    }

    async fn create_room(
        &self,
        service: &BareJid,
        request: CreateRoomType,
        sidebar_state: RoomSidebarState,
        behavior: CreateRoomBehavior,
    ) -> Result<Room, RoomError> {
        let availability = self
            .account_settings_repo
            .get(&self.ctx.connected_id()?.to_user_id())
            .await?
            .availability;
        let capabilities = &self.ctx.capabilities;

        let result = match request {
            CreateRoomType::Group { participants } => {
                self.create_or_join_group(
                    &service,
                    participants,
                    sidebar_state,
                    behavior,
                    capabilities,
                    availability,
                )
                .await
            }
            CreateRoomType::PrivateChannel { name } => {
                // We'll use a random ID for the jid of the private channel. This way
                // different people can create private channels with the same name without
                // creating a conflict. A conflict might also potentially be a security
                // issue if jid would contain sensitive information.
                let channel_id = self.id_provider.new_id();

                self.create_or_join_room_with_spec(
                    &service,
                    &format!("{}.{}", CHANNEL_PREFIX, channel_id),
                    &name,
                    RoomSpec::PrivateChannel,
                    sidebar_state,
                    behavior,
                    capabilities,
                    availability,
                    |_, _| async { Ok(()) },
                )
                .await
            }
            CreateRoomType::PublicChannel { name } => {
                // Prevent channels with duplicate names from being created…
                if !self.is_public_channel_name_unique(&name).await? {
                    return Err(RoomError::PublicChannelNameConflict);
                }

                // While it would be ideal to have channel names conflict, this could only
                // happen via its JID since this is the only thing that is unique. We do
                // have the requirement however that users should be able to rename their
                // channels, which is why they shouldn't conflict since the JIDs cannot be
                // changed after the fact. So we'll use a unique ID here as well.
                let channel_id = self.id_provider.new_id();

                self.create_or_join_room_with_spec(
                    &service,
                    &format!("{}.{}", CHANNEL_PREFIX, channel_id),
                    &name,
                    RoomSpec::PublicChannel,
                    sidebar_state,
                    behavior,
                    capabilities,
                    availability,
                    |_, _| async { Ok(()) },
                )
                .await
            }
        };

        let info = match result {
            Ok(metadata) => metadata,
            Err(RoomError::RoomIsAlreadyConnected(room_jid)) => {
                if let Some(room) = self.connected_rooms_repo.get(room_jid.as_ref()) {
                    return Ok(room);
                };
                return Err(RoomError::RoomIsAlreadyConnected(room_jid));
            }
            Err(error) => return Err(error),
        };

        self.finalize_pending_room(info).await
    }

    async fn create_or_join_group(
        &self,
        service: &BareJid,
        participants: Vec<UserId>,
        sidebar_state: RoomSidebarState,
        behavior: CreateRoomBehavior,
        capabilities: &Capabilities,
        availability: Availability,
    ) -> Result<RoomSessionInfo, RoomError> {
        if participants.len() < 2 {
            return Err(RoomError::InvalidNumberOfParticipants);
        }

        let user_jid = self.ctx.connected_id()?.into_user_id();

        // Load participant infos so that we can build a nice human-readable name for the group…
        let mut participant_names = vec![];
        let participants_including_self = participants
            .iter()
            .chain(iter::once(&user_jid))
            .cloned()
            .collect::<Vec<_>>();

        for jid in participants_including_self.iter() {
            let participant_name = self
                .user_profile_repo
                .get(jid)
                .await?
                .and_then(|profile| profile.first_name.or(profile.nickname))
                .unwrap_or_else(|| jid.username().to_uppercase_first_letter());
            participant_names.push(participant_name);
        }
        participant_names.sort();

        let group_name = participant_names.join(", ");

        // We'll create a hash of the sorted jids of our participants. This way users will always
        // come back to the exact same group if they accidentally try to create it again. Also
        // other participants (other than the creator of the room) are able to do the same without
        // having a bookmark.
        let group_hash = participants_including_self.group_name_hash();

        info!(
            "Trying to create group {} with participants {}",
            group_hash,
            participants_including_self
                .iter()
                .map(|jid| jid.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let info = self
            .create_or_join_room_with_spec(
                service,
                &group_hash,
                &group_name,
                RoomSpec::Group,
                sidebar_state,
                behavior,
                capabilities,
                availability,
                |room_id, info| {
                    // Try to promote all participants to owners…
                    info!("Update participant affiliations…");
                    let room_has_been_created = info.room_has_been_created;
                    let service = self.room_management_service.clone();

                    for participant in &participants {
                        if info.members.iter().find(|m| &m.id == participant).is_none() {
                            info.members.push(RoomSessionMember {
                                id: participant.clone(),
                                affiliation: RoomAffiliation::Owner,
                            });
                        }
                    }

                    let room_id = room_id.clone();
                    async move {
                        if room_has_been_created {
                            service.set_room_owners(&room_id, participants_including_self.as_slice()).await
                                .context(
                                    "Failed to update user affiliations of created group to type 'owner'",
                                )
                        } else {
                            Ok(())
                        }
                    }
                }
            )
            .await?;

        // Send invites…
        if info.room_has_been_created {
            info!("Sending invites for created group…");
            self.room_participation_service
                .invite_users_to_room(&info.room_id, participants.as_slice())
                .await?;
        }

        Ok(info)
    }

    async fn create_or_join_room_with_spec<Fut: Future<Output = Result<()>> + 'static>(
        &self,
        service: &BareJid,
        room_id: &str,
        room_name: &str,
        spec: RoomSpec,
        sidebar_state: RoomSidebarState,
        behavior: CreateRoomBehavior,
        capabilities: &Capabilities,
        availability: Availability,
        perform_additional_config: impl FnOnce(&MucId, &mut RoomSessionInfo) -> Fut,
    ) -> Result<RoomSessionInfo, RoomError> {
        let nickname = build_nickname(&self.ctx.connected_id()?.to_user_id());

        let mut attempt = 0;
        let mut unique_room_id = room_id.to_string();

        loop {
            let room_jid = MucId::from(BareJid::from_parts(
                Some(&NodePart::new(&unique_room_id)?),
                &service.domain(),
            ));
            let full_room_jid = room_jid.occupant_id_with_nickname(&nickname)?;

            // Insert pending room so that we don't miss any stanzas for this room while we're
            // creating (but potentially connecting to) it…
            self.insert_connecting_room(&room_jid, &nickname, sidebar_state)?;

            // Try to create or enter the room and configure it…
            let result = self
                .room_management_service
                .create_or_join_room(
                    &full_room_jid,
                    room_name,
                    spec.clone(),
                    capabilities,
                    availability,
                )
                .await;

            let mut info = match result {
                Ok(occupancy) => occupancy,
                Err(error) => {
                    // Remove pending room again…
                    self.connected_rooms_repo.delete(room_jid.as_ref());

                    // In this case the room existed in the past but was deleted. We'll modify
                    // the name and try again…
                    let Some(gone_error) = error.gone_err() else {
                        return Err(error.into());
                    };

                    match (behavior, gone_error.new_location) {
                        (CreateRoomBehavior::CreateUniqueIfGone, _)
                        | (CreateRoomBehavior::FollowThenCreateUnique, None) => {
                            unique_room_id = format!("{}#{}", room_id, attempt);
                            attempt += 1;
                            continue;
                        }
                        (CreateRoomBehavior::FollowIfGone, Some(new_location))
                        | (CreateRoomBehavior::FollowThenCreateUnique, Some(new_location)) => {
                            unique_room_id = new_location.to_string();
                            continue;
                        }
                        (CreateRoomBehavior::FailIfGone, _)
                        | (CreateRoomBehavior::FollowIfGone, None) => return Err(error.into()),
                    }
                }
            };

            match (perform_additional_config)(&room_jid, &mut info).await {
                Ok(_) => (),
                Err(error) => {
                    // Remove pending room again…
                    self.connected_rooms_repo.delete(room_jid.as_ref());
                    // Again, if the additional configuration fails and we've created the room
                    // we'll destroy it again.
                    if info.room_has_been_created {
                        _ = self
                            .room_management_service
                            .destroy_room(&room_jid, None)
                            .await;
                    }
                    return Err(error.into());
                }
            }

            return Ok(info);
        }
    }

    async fn finalize_pending_room(&self, info: RoomSessionInfo) -> Result<Room, RoomError> {
        // It could be the case that the room_jid was modified, i.e. if the preferred JID was
        // taken already.
        let room_name = info.config.room_name;
        let room_description = info.config.room_description;
        let room_topic = info.topic;
        let current_user_id = self.ctx.connected_id()?.into_user_id();

        let mut members = Vec::with_capacity(info.members.len());
        for member in info.members {
            let name = self
                .user_profile_repo
                .get_display_name(&member.id)
                .await
                .unwrap_or_default();
            let is_self = member.id == current_user_id;

            members.push(RegisteredMember {
                user_id: member.id,
                name,
                affiliation: member.affiliation,
                is_self,
            });
        }

        let room_info = RoomInfo {
            room_id: RoomId::Muc(info.room_id.clone()),
            user_nickname: info.user_nickname,
            r#type: info.config.room_type,
            features: info.config.features,
        };

        let Some(room) = self.connected_rooms_repo.update(info.room_id.as_ref(), {
            let room_name = room_name;
            Box::new(move |room| {
                // Convert the temporary room to its final form…
                let room = room.by_resolving_with_info(
                    room_name,
                    room_description,
                    room_topic,
                    room_info,
                    members,
                    info.participants,
                );
                room
            })
        }) else {
            return Err(RoomError::RoomWasModified);
        };

        Ok(room)
    }

    async fn is_public_channel_name_unique(&self, channel_name: &str) -> Result<bool> {
        let available_rooms = self
            .room_management_service
            .load_public_rooms(&self.ctx.muc_service()?)
            .await?;

        let lowercase_channel_name = channel_name.to_lowercase();
        for room in available_rooms {
            let Some(mut room_name) = room.name else {
                continue;
            };
            room_name.make_ascii_lowercase();
            if room_name == lowercase_channel_name {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn insert_connecting_room(
        &self,
        room_id: &MucId,
        nickname: &str,
        sidebar_state: RoomSidebarState,
    ) -> Result<Room, RoomError> {
        let room_id = RoomId::Muc(room_id.clone());

        // If we have a pending room waiting for us, we'll switch that to connecting and do not
        // insert a new one.
        if let Some(pending_room) = self.connected_rooms_repo.get(room_id.as_ref()) {
            if pending_room.state() == RoomState::Pending {
                pending_room.set_state(RoomState::Connecting);
                return Ok(pending_room);
            }
        }

        let room = Room::connecting(&room_id, nickname, sidebar_state);
        self.connected_rooms_repo
            .set(room.clone())
            .map_err(|_| RoomError::RoomIsAlreadyConnected(room_id))?;
        Ok(room)
    }
}
