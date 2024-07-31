// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::future::Future;
use std::iter;
use std::ops::Deref;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use jid::{BareJid, NodePart};
use tracing::{debug, error, info, warn};
use xmpp_parsers::stanza_error::DefinedCondition;

use prose_proc_macros::DependenciesStruct;
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::{IDProvider, RequestError};

use crate::app::deps::{
    DynAccountSettingsRepository, DynAppContext, DynClientEventDispatcher,
    DynConnectedRoomsRepository, DynEncryptionDomainService, DynIDProvider,
    DynMessageArchiveDomainService, DynMessageMigrationDomainService, DynRoomAttributesService,
    DynRoomManagementService, DynRoomParticipationService, DynSyncedRoomSettingsService,
    DynUserInfoDomainService,
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
use crate::domain::settings::models::SyncedRoomSettings;
use crate::domain::shared::models::{AccountId, CachePolicy, MucId, RoomId, RoomType, UserId};
use crate::domain::user_info::models::{Presence, UserInfoOptExt};
use crate::dtos::{Availability, RoomState};
use crate::{ClientEvent, ClientRoomEventType};

use super::super::{CreateRoomType, RoomsDomainService as RoomsDomainServiceTrait};
use super::{build_nickname, ParticipantsVecExt};

const CHANNEL_PREFIX: &str = "org.prose.channel";

#[derive(DependenciesStruct)]
pub struct RoomsDomainService {
    account_settings_repo: DynAccountSettingsRepository,
    client_event_dispatcher: DynClientEventDispatcher,
    connected_rooms_repo: DynConnectedRoomsRepository,
    ctx: DynAppContext,
    encryption_domain_service: DynEncryptionDomainService,
    id_provider: DynIDProvider,
    message_archive_domain_service: DynMessageArchiveDomainService,
    message_migration_domain_service: DynMessageMigrationDomainService,
    room_attributes_service: DynRoomAttributesService,
    room_management_service: DynRoomManagementService,
    room_participation_service: DynRoomParticipationService,
    synced_room_settings_service: DynSyncedRoomSettingsService,
    user_info_domain_service: DynUserInfoDomainService,
}

/// Represents the outcome of attempting to fetch or create a Room in the `ConnectedRoomsRepository`.
///
/// This enum indicates whether the Room was found to exist in the repository before the operation
/// or if it had to be created. Note that this does not reflect whether the room was created on the
/// server. For server creation status, refer to `RoomSessionInfo.room_has_been_created`.
enum RoomStatus {
    /// The Room did not exist in the repository and was newly created.
    IsNew(Room),
    /// The Room already existed in the repository.
    Exists(Room),
}

impl RoomStatus {
    fn is_new(&self) -> bool {
        match self {
            RoomStatus::IsNew(_) => true,
            RoomStatus::Exists(_) => false,
        }
    }
}

impl From<RoomStatus> for Room {
    fn from(value: RoomStatus) -> Self {
        match value {
            RoomStatus::IsNew(room) => room,
            RoomStatus::Exists(room) => room,
        }
    }
}

impl Deref for RoomStatus {
    type Target = Room;

    fn deref(&self) -> &Self::Target {
        match self {
            RoomStatus::IsNew(room) => room,
            RoomStatus::Exists(room) => room,
        }
    }
}

/// Used to create a RoomStatus
enum RoomInfoStatus {
    IsNew(RoomSessionInfo),
    Exists(RoomSessionInfo),
}

impl Deref for RoomInfoStatus {
    type Target = RoomSessionInfo;

    fn deref(&self) -> &Self::Target {
        match self {
            RoomInfoStatus::IsNew(info) => info,
            RoomInfoStatus::Exists(info) => info,
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl RoomsDomainServiceTrait for RoomsDomainService {
    async fn create_or_join_room(
        &self,
        request: CreateOrEnterRoomRequest,
        sidebar_state: RoomSidebarState,
    ) -> Result<Room, RoomError> {
        let account = self.ctx.connected_account()?;

        let (room, context) = match request {
            CreateOrEnterRoomRequest::Create {
                service,
                room_type,
                behavior,
                decryption_context,
            } => (
                self.create_room(&account, &service, room_type, sidebar_state, behavior)
                    .await?,
                decryption_context,
            ),
            CreateOrEnterRoomRequest::JoinRoom {
                room_id,
                password,
                behavior,
                decryption_context,
            } => (
                self.join_room(
                    &account,
                    &room_id,
                    password.as_deref(),
                    sidebar_state,
                    behavior,
                )
                .await?,
                decryption_context,
            ),
            CreateOrEnterRoomRequest::JoinDirectMessage {
                participant,
                decryption_context,
            } => (
                self.join_direct_message(&account, &participant, sidebar_state)
                    .await?,
                decryption_context,
            ),
        };

        let needs_finalize_context = context.is_none();
        let context = context.unwrap_or_default();

        match self
            .message_archive_domain_service
            .catchup_room(&room, context.clone())
            .await
        {
            Ok(new_messages_found) => {
                // If the room existed before, and we found new messages notify clients so that
                // they can reload the messages.
                if !room.is_new() && new_messages_found {
                    self.client_event_dispatcher
                        .dispatch_room_event(room.clone(), ClientRoomEventType::MessagesNeedReload);
                }
            }
            Err(err) => {
                error!("Failed to catch up room. {}", err.to_string())
            }
        }

        if needs_finalize_context {
            self.encryption_domain_service
                .finalize_decryption(context)
                .await;
        }

        Ok(room.into())
    }

    async fn reconnect_room_if_needed(&self, room_id: &MucId) -> Result<(), RoomError> {
        let account = self.ctx.connected_account()?;
        let room = self
            .connected_rooms_repo
            .get(&account, room_id.as_ref())
            .ok_or(RoomError::RoomNotFound)?;

        let occupant_id = room
            .occupant_id()
            .expect("A MUC room must have an OccupantId");

        match room.state() {
            RoomState::Pending
            | RoomState::Connecting
            | RoomState::Disconnected {
                can_retry: false, ..
            } => {
                // If the room is in the process of connecting or disconnected with no
                // chance of retrying, we'll leave it alone.
                return Ok(());
            }
            RoomState::Disconnected {
                can_retry: true, ..
            } => {
                // If the room was already disconnected, we'll try to reconnect it…
                self.reconnect_room(room_id, room).await?;
                return Ok(());
            }
            RoomState::Connected => (),
        }

        info!("Sending self-ping to {room_id}");

        let Err(err) = self
            .room_management_service
            .send_self_ping(&occupant_id)
            .await
        else {
            // The self-ping succeeded. We're still connected.
            info!("{room_id} is still connected.");
            return Ok(());
        };

        // https://xmpp.org/extensions/xep-0410.html#performingselfping
        match err {
            RequestError::Disconnected => {
                room.set_state(RoomState::Disconnected {
                    error: None,
                    can_retry: true,
                });
                return Ok(());
            }

            RequestError::TimedOut => {
                info!("Ping to {room_id} timed out.");
                // The MUC service (or another client) is unreachable. The client may indicate
                // the status to the user and re-attempt the self-ping after some timeout,
                // until it receives either an error or a success response.
                room.set_state(RoomState::Disconnected {
                    error: None,
                    can_retry: true,
                });
                return Ok(());
            }

            RequestError::XMPP { .. }
                if err.defined_condition() == Some(DefinedCondition::RemoteServerNotFound)
                    || err.defined_condition() == Some(DefinedCondition::RemoteServerTimeout) =>
            {
                // The remote server is unreachable for unspecified reasons; this can be a
                // temporary network failure or a server outage. No decision can be made based
                // on this; Treat like a timeout
                info!("{room_id} is unreachable.");
                room.set_state(RoomState::Disconnected {
                    error: None,
                    can_retry: true,
                });
                return Ok(());
            }

            RequestError::XMPP { .. }
                if err.defined_condition() == Some(DefinedCondition::ServiceUnavailable)
                    || err.defined_condition() == Some(DefinedCondition::FeatureNotImplemented) =>
            {
                // The client is joined, but the pinged client does not implement XMPP Ping (XEP-0199).
                info!("{room_id} is still connected (but doesn't support XMPP Ping).");
                return Ok(());
            }

            RequestError::XMPP { .. }
                if err.defined_condition() == Some(DefinedCondition::ItemNotFound) =>
            {
                // the client is joined, but the occupant just changed their name
                // (e.g. initiated by a different client).
                info!("{room_id} is still connected.");
                return Ok(());
            }

            _ => {
                // Any other error [4]: the client is probably not joined anymore. It should
                // perform a re-join.
            }
        }

        info!("{room_id} is not connected anymore.");
        self.reconnect_room(&room_id, room).await?;
        Ok(())
    }

    /// Renames the room identified by `room_jid` to `name`.
    ///
    /// - If the room is not connected, no action is performed.
    /// - Panics if the Room is not of type `RoomType::PublicChannel`, `RoomType::PrivateChannel`
    ///   or `RoomType::Generic`.
    /// - Fails with `RoomError::PublicChannelNameConflict` if the room is of type
    ///   `RoomType::PublicChannel` and `name` is already used by another public channel.
    async fn rename_room(&self, room_id: &MucId, name: &str) -> Result<(), RoomError> {
        let Some(room) = self
            .connected_rooms_repo
            .get(&self.ctx.connected_account()?, room_id.as_ref())
        else {
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
    /// If the room is not connected, no action is performed, otherwise:
    /// - Panics if the reconfiguration is not allowed. Allowed reconfigurations are:
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
        let account = self.ctx.connected_account()?;

        let Some(room) = self.connected_rooms_repo.get(&account, room_id.as_ref()) else {
            return Err(RoomError::RoomNotFound);
        };

        match (&room.r#type, spec.room_type()) {
            (RoomType::Group, RoomType::PrivateChannel) => {
                // Remove room first so that we don't run into problems with reentrancy…
                self.connected_rooms_repo.delete(&account, room_id.as_ref());

                let service = BareJid::from_parts(None, &room_id.domain());

                // Create new room
                debug!("Creating new room {}…", new_name);
                let new_room = match self
                    .create_room(
                        &account,
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
                        _ = self.connected_rooms_repo.set(&account, room);
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
                        _ = self.connected_rooms_repo.set(&account, room);
                        _ = self
                            .room_management_service
                            .destroy_room(new_room_id, None)
                            .await;
                        return Err(err.into());
                    }
                }

                let current_user = self.ctx.connected_id()?.to_user_id();
                let member_ids = room.with_participants(|p| {
                    p.values()
                        .filter_map(|p| {
                            if p.affiliation >= RoomAffiliation::Member {
                                return p.real_id.clone();
                            }
                            None
                        })
                        .collect::<Vec<_>>()
                });

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

                Ok(new_room.into())
            }
            (RoomType::PrivateChannel, RoomType::PublicChannel) => {
                // Ensure that the new name doesn't exist already.
                if !self.is_public_channel_name_unique(new_name).await? {
                    return Err(RoomError::PublicChannelNameConflict);
                }

                self.room_management_service
                    .reconfigure_room(room_id, spec, new_name)
                    .await?;

                let Some(room) = self
                    .connected_rooms_repo
                    .update(&account, room_id.as_ref(), {
                        Box::new(|room| room.by_changing_type(RoomType::PublicChannel))
                    })
                else {
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

                let Some(room) = self
                    .connected_rooms_repo
                    .update(&account, room_id.as_ref(), {
                        Box::new(|room| room.by_changing_type(RoomType::PrivateChannel))
                    })
                else {
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
        let account = self.ctx.connected_account()?;

        let Some(room) = self.connected_rooms_repo.get(&account, room_id.as_ref()) else {
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
                &account,
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
        account: &AccountId,
        room_id: &MucId,
        password: Option<&str>,
        sidebar_state: RoomSidebarState,
        behavior: JoinRoomBehavior,
    ) -> Result<RoomStatus, RoomError> {
        let remove_or_retain_room_on_error =
            |room: Room, error: &RoomError| match behavior.on_failure {
                JoinRoomFailureBehavior::RemoveOnError => {
                    self.connected_rooms_repo.delete(account, room_id.as_ref());
                }
                JoinRoomFailureBehavior::RetainOnError => room.set_state(RoomState::Disconnected {
                    error: Some(error.to_string()),
                    can_retry: true,
                }),
            };

        let display_name = self
            .user_info_domain_service
            .get_user_info(account.as_ref(), CachePolicy::ReturnCacheDataElseLoad)
            .await?
            .display_name()
            .unwrap_or_username(account.as_ref());

        let nickname = build_nickname(Some(&display_name), account.as_ref());
        let mut room_id = room_id.clone();
        let availability = self.account_settings_repo.get(&account).await?.availability;
        let capabilities = &self.ctx.capabilities;
        let password = password.map(ToString::to_string);

        let info = 'info: loop {
            // Insert pending room so that we don't miss any stanzas for this room while we're
            // connecting to it…
            let room = self.insert_connecting_room(account, &room_id, &nickname, sidebar_state)?;

            let join_room = {
                let room_id = room_id.clone();
                let password = password.clone();
                let display_name = display_name.clone();

                move |nickname| {
                    let password = password.clone();
                    let display_name = display_name.clone();
                    let full_room_jid = room_id.occupant_id_with_nickname(&nickname);

                    async move {
                        self.room_management_service
                            .join_room(
                                &full_room_jid?,
                                password.as_deref(),
                                &display_name,
                                capabilities,
                                availability,
                            )
                            .await
                    }
                }
            };

            let result = Self::try_until_nickname_unique(&nickname, join_room, 10).await;

            match result {
                Ok(info) => {
                    break 'info if room.is_new() {
                        RoomInfoStatus::IsNew(info)
                    } else {
                        RoomInfoStatus::Exists(info)
                    }
                }
                Err(error) => {
                    let Some(gone_error) = error.gone_err() else {
                        remove_or_retain_room_on_error(room.into(), &error);
                        return Err(error);
                    };

                    match (behavior.on_redirect, gone_error.new_location) {
                        (JoinRoomRedirectBehavior::FollowIfGone, Some(new_location)) => {
                            self.connected_rooms_repo.delete(account, room_id.as_ref());
                            room_id = new_location;
                            continue;
                        }
                        (JoinRoomRedirectBehavior::FollowIfGone, None)
                        | (JoinRoomRedirectBehavior::FailIfGone, _) => {
                            remove_or_retain_room_on_error(room.into(), &error);
                            return Err(error);
                        }
                    }
                }
            };
        };

        self.finalize_pending_room(account, info).await
    }

    async fn join_direct_message(
        &self,
        account: &AccountId,
        user_id: &UserId,
        sidebar_state: RoomSidebarState,
    ) -> Result<RoomStatus, RoomError> {
        let (room_is_new, existing_room) = match self
            .connected_rooms_repo
            .get(account, user_id.as_ref())
        {
            Some(room) if room.state() == RoomState::Pending || room.state().is_disconnected() => {
                (false, Some(room))
            }
            None => (true, None),
            Some(room) => return Ok(RoomStatus::Exists(room)),
        };

        let user_info = self
            .user_info_domain_service
            .get_user_info(user_id, CachePolicy::ReturnCacheDataElseLoad)
            .await
            .unwrap_or_default()
            .unwrap_or_default();
        let contact_name = user_info.display_name().unwrap_or_username(user_id);

        let room_id = RoomId::User(user_id.clone());
        let settings = self
            .synced_room_settings_service
            .load_settings(&room_id)
            .await
            .unwrap_or_default()
            .or(existing_room.map(|room| room.settings()))
            .unwrap_or_else(|| SyncedRoomSettings::new(room_id));

        let features = self.ctx.server_features()?;

        let room = Room::for_direct_message(
            &user_id,
            &contact_name,
            Presence {
                availability: user_info.availability,
                avatar: user_info.avatar,
                caps: user_info.caps,
                client: user_info.client,
                nickname: None,
                priority: 0,
                status: None,
            },
            sidebar_state,
            RoomFeatures {
                mam_version: features.mam_version,
                server_time_offset: features.server_time_offset,
                self_ping_optimization: false,
            },
            settings,
        );

        self.connected_rooms_repo
            .set_or_replace(account, room.clone());

        let status = if room_is_new {
            RoomStatus::IsNew(room)
        } else {
            RoomStatus::Exists(room)
        };

        Ok(status)
    }

    async fn create_room(
        &self,
        account: &AccountId,
        service: &BareJid,
        request: CreateRoomType,
        sidebar_state: RoomSidebarState,
        behavior: CreateRoomBehavior,
    ) -> Result<RoomStatus, RoomError> {
        let availability = self.account_settings_repo.get(account).await?.availability;
        let capabilities = &self.ctx.capabilities;

        let result = match request {
            CreateRoomType::Group { participants } => {
                self.create_or_join_group(
                    account,
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
                    account,
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
                    account,
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
                if let Some(room) = self.connected_rooms_repo.get(account, room_jid.as_ref()) {
                    return Ok(RoomStatus::Exists(room));
                };
                return Err(RoomError::RoomIsAlreadyConnected(room_jid));
            }
            Err(error) => return Err(error),
        };

        self.finalize_pending_room(account, info).await
    }

    async fn create_or_join_group(
        &self,
        account: &AccountId,
        service: &BareJid,
        participants: Vec<UserId>,
        sidebar_state: RoomSidebarState,
        behavior: CreateRoomBehavior,
        capabilities: &Capabilities,
        availability: Availability,
    ) -> Result<RoomInfoStatus, RoomError> {
        if participants.len() < 2 {
            return Err(RoomError::InvalidNumberOfParticipants);
        }

        // Load participant infos so that we can build a nice human-readable name for the group…
        let mut participant_names = vec![];
        let participants_including_self = participants
            .iter()
            .chain(iter::once(account.as_ref()))
            .cloned()
            .collect::<Vec<_>>();

        for user_id in participants_including_self.iter() {
            let participant_name = self
                .user_info_domain_service
                .get_user_info(user_id, CachePolicy::ReturnCacheDataElseLoad)
                .await
                .unwrap_or_default()
                .display_name()
                .unwrap_or_username(user_id);
            participant_names.push(participant_name);
        }
        participant_names.sort();

        let group_name = participant_names.join(", ");

        // We'll create a hash of the sorted jids of our participants. This way users will always
        // come back to the exact same group if they accidentally try to create it again. Also,
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
                account,
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
        account: &AccountId,
        service: &BareJid,
        room_id: &str,
        room_name: &str,
        spec: RoomSpec,
        sidebar_state: RoomSidebarState,
        behavior: CreateRoomBehavior,
        capabilities: &Capabilities,
        availability: Availability,
        perform_additional_config: impl FnOnce(&MucId, &mut RoomSessionInfo) -> Fut,
    ) -> Result<RoomInfoStatus, RoomError> {
        let display_name = self
            .user_info_domain_service
            .get_user_info(account.as_ref(), CachePolicy::ReturnCacheDataElseLoad)
            .await?
            .display_name()
            .unwrap_or_username(account.as_ref());
        let nickname = build_nickname(Some(&display_name), account.as_ref());

        let mut attempt = 0;
        let mut unique_room_id = room_id.to_string();

        loop {
            let room_jid = MucId::from(BareJid::from_parts(
                Some(&NodePart::new(&unique_room_id)?),
                &service.domain(),
            ));

            // Insert pending room so that we don't miss any stanzas for this room while we're
            // creating (but potentially connecting to) it…
            let room_is_new = self
                .insert_connecting_room(account, &room_jid, &nickname, sidebar_state)?
                .is_new();

            let create_or_join_room = {
                let room_jid = room_jid.clone();
                let display_name = display_name.clone();
                let spec = spec.clone();

                move |nickname| {
                    let full_room_jid = room_jid.occupant_id_with_nickname(&nickname);
                    let display_name = display_name.clone();
                    let spec = spec.clone();

                    async move {
                        self.room_management_service
                            .create_or_join_room(
                                &full_room_jid?,
                                room_name,
                                &display_name,
                                spec,
                                capabilities,
                                availability,
                            )
                            .await
                    }
                }
            };

            // Try to create or enter the room and configure it…
            let result = Self::try_until_nickname_unique(&nickname, create_or_join_room, 10).await;

            let mut info = match result {
                Ok(occupancy) => occupancy,
                Err(error) => {
                    // Remove pending room again…
                    self.connected_rooms_repo.delete(account, room_jid.as_ref());

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

            match perform_additional_config(&room_jid, &mut info).await {
                Ok(_) => (),
                Err(error) => {
                    // Remove pending room again…
                    self.connected_rooms_repo.delete(account, room_jid.as_ref());
                    // Again, if the additional configuration fails, and we've created the room
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

            let status = if room_is_new {
                RoomInfoStatus::IsNew(info)
            } else {
                RoomInfoStatus::Exists(info)
            };

            return Ok(status);
        }
    }

    async fn reconnect_room(&self, room_id: &MucId, room: Room) -> Result<(), RoomError> {
        info!("Reconnecting {room_id}…");

        room.set_state(RoomState::Pending);

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        self.create_or_join_room(
            CreateOrEnterRoomRequest::JoinRoom {
                room_id: room_id.clone(),
                password: None,
                behavior: JoinRoomBehavior {
                    on_redirect: JoinRoomRedirectBehavior::FollowIfGone,
                    on_failure: JoinRoomFailureBehavior::RetainOnError,
                },
                decryption_context: None,
            },
            room.sidebar_state(),
        )
        .await?;

        Ok(())
    }

    async fn finalize_pending_room(
        &self,
        account: &AccountId,
        info: RoomInfoStatus,
    ) -> Result<RoomStatus, RoomError> {
        let (info, room_is_new) = match info {
            RoomInfoStatus::IsNew(info) => (info, true),
            RoomInfoStatus::Exists(info) => (info, false),
        };

        // It could be the case that the room_jid was modified, i.e. if the preferred JID was
        // taken already.
        let room_name = info.config.room_name;
        let room_description = info.config.room_description;
        let room_topic = info.topic;
        let current_user_id = self.ctx.connected_id()?.into_user_id();

        // Enrich the room members with vCard data…
        let mut members = Vec::with_capacity(info.members.len());
        for member in info.members {
            let name = self
                .user_info_domain_service
                .get_user_info(&member.id, CachePolicy::ReturnCacheDataElseLoad)
                .await
                .unwrap_or_default()
                .display_name()
                .build();
            let is_self = member.id == current_user_id;

            members.push(RegisteredMember {
                user_id: member.id,
                name,
                affiliation: member.affiliation,
                is_self,
            });
        }

        let room_id = RoomId::Muc(info.room_id.clone());

        // TODO: If the MUC room is not on our server, determine the time offset of that server.
        let server_time_offset = self
            .ctx
            .is_muc_room_on_connected_server(&info.room_id)
            .then(|| {
                self.ctx
                    .server_features()
                    .map(|f| f.server_time_offset)
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        let room_info = RoomInfo {
            room_id: room_id.clone(),
            user_nickname: info.user_nickname,
            r#type: info.config.room_type,
            features: RoomFeatures {
                mam_version: info.config.mam_version,
                server_time_offset,
                self_ping_optimization: info.config.supports_self_ping_optimization,
            },
        };

        let settings = self
            .synced_room_settings_service
            .load_settings(&room_id)
            .await
            .unwrap_or_default()
            .unwrap_or_else(|| SyncedRoomSettings::new(room_id));

        let Some(room) = self
            .connected_rooms_repo
            .update(account, info.room_id.as_ref(), {
                let room_name = room_name;
                Box::new(move |room| {
                    if !room.is_connecting() {
                        warn!("Not resolving freshly connected room, since it has been modified in the meanwhile. Current state is {:?}", room.state());
                        return room;
                    }

                    // Convert the temporary room to its final form…
                    let room = room.by_resolving_with_info(
                        room_name,
                        room_description,
                        room_topic,
                        room_info,
                        members,
                        info.participants,
                        settings,
                    );
                    room
                })
            })
        else {
            return Err(RoomError::RoomWasModified);
        };

        let status = if room_is_new {
            RoomStatus::IsNew(room)
        } else {
            RoomStatus::Exists(room)
        };

        Ok(status)
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
        account: &AccountId,
        room_id: &MucId,
        nickname: &str,
        sidebar_state: RoomSidebarState,
    ) -> Result<RoomStatus, RoomError> {
        let room_id = RoomId::Muc(room_id.clone());

        // If we have a pending room waiting for us, we'll switch that to connecting and do not
        // insert a new one.
        if let Some(pending_room) = self.connected_rooms_repo.get(account, room_id.as_ref()) {
            if pending_room.state() == RoomState::Pending || pending_room.state().is_disconnected()
            {
                pending_room.set_state(RoomState::Connecting);
                return Ok(RoomStatus::Exists(pending_room));
            }
        }

        let room = Room::connecting(&room_id, nickname, sidebar_state);
        self.connected_rooms_repo
            .set(account, room.clone())
            .map_err(|_| RoomError::RoomIsAlreadyConnected(room_id))?;
        Ok(RoomStatus::IsNew(room))
    }

    /// Attempts to join a chat room with a unique nickname, retrying with modified nicknames upon conflicts.
    ///
    /// This function takes a preferred nickname and tries to use it to join a chat room. If the nickname
    /// is already in use (indicated by a `RoomError::RequestError` with a `Conflict` condition), it appends
    /// a numerical suffix (e.g., `nickname#1`, `nickname#2`, etc.) and retries the operation. This process
    /// repeats until a unique nickname is secured or the maximum number of attempts is reached.
    ///
    /// # Parameters
    /// - `preferred_nickname`: The base nickname preferred by the user. This is the starting point for any modifications.
    /// - `join_handler`: A closure that attempts to join the chat room with a given nickname. It should return
    ///   `Ok(T)` on successful joining, or `Err(RoomError)` if there is an issue, such as a nickname conflict.
    /// - `max_attempts`: The maximum number of attempts to make. If this limit is reached, the function
    ///   will return the last encountered `RoomError`. Set to `0` for unlimited attempts.
    ///
    /// # Returns
    /// Returns `Ok(T)` if joining the chat room was successful with a unique nickname. If the maximum number
    /// of attempts is reached or another error occurs, returns `Err(RoomError)`.
    ///
    /// # Errors
    /// This function propagates `RoomError`s from the `join_handler`. If a `RoomError::RequestRequest` with a
    /// `Conflict` condition is not resolved after `max_attempts`, the function returns this error.
    async fn try_until_nickname_unique<F, T, U>(
        preferred_nickname: &str,
        mut join_handler: F,
        max_attempts: u32,
    ) -> Result<U, RoomError>
    where
        F: FnMut(String) -> T + SendUnlessWasm + SyncUnlessWasm,
        T: Future<Output = Result<U, RoomError>> + SendUnlessWasm,
    {
        let mut attempt = 0;

        loop {
            let nickname = if attempt == 0 {
                preferred_nickname.to_string()
            } else {
                format!("{}#{}", preferred_nickname, attempt)
            };
            attempt += 1;

            match join_handler(nickname).await {
                Ok(result) => return Ok(result),
                Err(RoomError::RequestError(error))
                    if error.defined_condition() == Some(DefinedCondition::Conflict) =>
                {
                    if max_attempts > 0 && attempt >= max_attempts {
                        return Err(RoomError::RequestError(error));
                    }
                }
                Err(err) => return Err(err),
            }
        }
    }
}
