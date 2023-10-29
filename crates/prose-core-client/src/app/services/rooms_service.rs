// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::future::Future;
use std::iter;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use jid::{BareJid, NodePart};
use parking_lot::RwLock;
use sha1::{Digest, Sha1};
use tracing::{error, info};
use xmpp_parsers::stanza_error::DefinedCondition;

use prose_proc_macros::InjectDependencies;
use prose_wasm_utils::PinnedFuture;
use prose_xmpp::mods::muc::RoomConfigResponse;
use prose_xmpp::{mods, RequestError};

use crate::app::deps::{
    DynAppContext, DynAppServiceDependencies, DynBookmarksRepository, DynConnectedRoomsRepository,
    DynRoomFactory, DynRoomManagementService, DynRoomParticipationService, DynUserProfileService,
};
use crate::app::services::RoomEnvelope;
use crate::domain::rooms::models::{Bookmark, RoomConfig, RoomError, RoomInternals, RoomMetadata};
use crate::util::StringExt;
use crate::ClientEvent;

#[derive(InjectDependencies)]
pub struct RoomsService {
    #[inject]
    connected_rooms_repo: DynConnectedRoomsRepository,
    #[inject]
    room_management_service: DynRoomManagementService,
    #[inject]
    user_profile_service: DynUserProfileService,
    #[inject]
    room_participation_service: DynRoomParticipationService,
    #[inject]
    room_factory: DynRoomFactory,
    #[inject]
    bookmarks_repo: DynBookmarksRepository,
    #[inject]
    ctx: DynAppContext,
    #[inject]
    app_service: DynAppServiceDependencies,
}

impl RoomsService {
    pub fn connected_rooms(&self) -> Vec<RoomEnvelope> {
        self.connected_rooms_repo
            .get_all()
            .into_iter()
            .filter_map(|internals| {
                if internals.is_pending() {
                    None
                } else {
                    Some(self.room_factory.build(internals.clone()))
                }
            })
            .collect()
    }

    pub async fn load_public_rooms(&self) -> Result<Vec<mods::muc::Room>> {
        Ok(self
            .room_management_service
            .load_public_rooms(&self.ctx.muc_service()?)
            .await?)
    }

    pub async fn join_room(
        &self,
        room_jid: &BareJid,
        password: Option<&str>,
    ) -> Result<RoomEnvelope> {
        let room = self
            .create_or_join_room(CreateOrEnterRoomRequest {
                r#type: CreateOrEnterRoomRequestType::Join {
                    room_jid: room_jid.clone(),
                    nickname: None,
                    password: password.map(ToString::to_string),
                },
                save_bookmark: true,
                notify_delegate: false,
            })
            .await?;
        Ok(room)
    }

    pub async fn create_room_for_direct_message(
        &self,
        participants: &[BareJid],
    ) -> Result<RoomEnvelope> {
        if participants.is_empty() {
            bail!("Group must have at least one other participant.")
        }

        let room = self
            .create_or_join_room(CreateOrEnterRoomRequest {
                r#type: CreateOrEnterRoomRequestType::Create {
                    service: self.ctx.muc_service()?,
                    room_type: CreateRoomType::Group {
                        participants: participants.to_vec(),
                        send_invites: true,
                    },
                },
                save_bookmark: true,
                notify_delegate: false,
            })
            .await?;

        Ok(room)
    }

    pub async fn create_room_for_private_channel(
        &self,
        channel_name: impl AsRef<str>,
    ) -> Result<RoomEnvelope> {
        // Create room…
        info!(
            "Creating private channel with name {}…",
            channel_name.as_ref()
        );

        let room = self
            .create_or_join_room(CreateOrEnterRoomRequest {
                r#type: CreateOrEnterRoomRequestType::Create {
                    service: self.ctx.muc_service()?,
                    room_type: CreateRoomType::PrivateChannel {
                        name: channel_name.as_ref().to_string(),
                    },
                },
                save_bookmark: true,
                notify_delegate: false,
            })
            .await?;

        Ok(room)
    }

    pub async fn create_room_for_public_channel(
        &self,
        channel_name: impl AsRef<str>,
    ) -> Result<RoomEnvelope> {
        // Create room…
        info!(
            "Creating public channel with name {}…",
            channel_name.as_ref()
        );

        let room = self
            .create_or_join_room(CreateOrEnterRoomRequest {
                r#type: CreateOrEnterRoomRequestType::Create {
                    service: self.ctx.muc_service()?,
                    room_type: CreateRoomType::PublicChannel {
                        name: channel_name.as_ref().to_string(),
                    },
                },
                save_bookmark: true,
                notify_delegate: false,
            })
            .await?;

        Ok(room)
    }

    pub async fn destroy_room(&self, room_jid: &BareJid) -> Result<()> {
        self.room_management_service.destroy_room(room_jid).await?;
        Ok(())
    }
}

pub(crate) enum CreateRoomType {
    Group {
        participants: Vec<BareJid>,
        send_invites: bool,
    },
    PrivateChannel {
        name: String,
    },
    PublicChannel {
        name: String,
    },
}

pub(crate) enum CreateOrEnterRoomRequestType {
    Create {
        service: BareJid,
        room_type: CreateRoomType,
    },
    Join {
        room_jid: BareJid,
        nickname: Option<String>,
        password: Option<String>,
    },
}

pub(crate) struct CreateOrEnterRoomRequest {
    pub r#type: CreateOrEnterRoomRequestType,
    pub save_bookmark: bool,
    pub notify_delegate: bool,
}

const GROUP_PREFIX: &str = "org.prose.group";
const PRIVATE_CHANNEL_PREFIX: &str = "org.prose.private-channel";
const PUBLIC_CHANNEL_PREFIX: &str = "org.prose.public-channel";

impl RoomsService {
    pub(crate) async fn create_or_join_room(
        &self,
        CreateOrEnterRoomRequest {
            r#type,
            save_bookmark,
            notify_delegate,
        }: CreateOrEnterRoomRequest,
    ) -> Result<RoomEnvelope, RoomError> {
        let user_jid = self
            .ctx
            .connected_jid()
            .map_err(|err| RequestError::Generic {
                msg: err.to_string(),
            })?
            .into_bare();
        let default_nickname = user_jid.node_str().unwrap_or("unknown-user");

        let result = match r#type {
            CreateOrEnterRoomRequestType::Create { service, room_type } => match room_type {
                CreateRoomType::Group {
                    participants,
                    send_invites,
                } => {
                    self.create_or_join_group(
                        &service,
                        &user_jid,
                        default_nickname,
                        participants,
                        send_invites,
                    )
                    .await
                }
                CreateRoomType::PrivateChannel { name } => {
                    // We'll use a random ID for the jid of the private channel. This way
                    // different people can create private channels with the same name without
                    // creating a conflict. A conflict might also potentially be a security
                    // issue if jid would contain sensitive information.
                    let channel_id = self.app_service.id_provider.new_id();

                    self.create_or_join_room_with_config(
                        &service,
                        &user_jid,
                        &format!("{}.{}", PRIVATE_CHANNEL_PREFIX, channel_id),
                        default_nickname,
                        RoomConfig::private_channel(&name),
                        |_| async { Ok(()) },
                    )
                    .await
                }
                CreateRoomType::PublicChannel { name } => {
                    // Public channels should be able to conflict, i.e. there should only be
                    // one channel for any given name. Since these can be discovered publicly
                    // and joined by anyone there should be no harm in exposing the name in
                    // the jid.

                    self.create_or_join_room_with_config(
                        &service,
                        &user_jid,
                        &format!(
                            "{}.{}",
                            PUBLIC_CHANNEL_PREFIX,
                            name.to_ascii_lowercase().replace(" ", "-")
                        ),
                        default_nickname,
                        RoomConfig::public_channel(&name),
                        |_| async { Ok(()) },
                    )
                    .await
                }
            },
            CreateOrEnterRoomRequestType::Join {
                room_jid,
                nickname,
                password,
            } => {
                self.join_room_by_resolving_nickname_conflict(
                    &room_jid,
                    &user_jid,
                    nickname.as_deref().unwrap_or(default_nickname),
                    password.as_deref(),
                    0,
                )
                .await
            }
        };

        let metadata = match result {
            Ok(metadata) => metadata,
            Err(RoomError::RoomIsAlreadyConnected(room_jid)) => {
                if let Some(room) = self.connected_rooms_repo.get(&room_jid) {
                    return Ok(self.room_factory.build(room));
                };
                return Err(RoomError::RoomIsAlreadyConnected(room_jid));
            }
            Err(error) => return Err(error),
        };

        // It could be the case that the room_jid was modified, i.e. if the preferred JID was
        // taken already.
        let room_jid = metadata.room_jid.clone();

        let Some(room) = self.connected_rooms_repo.update(
            &room_jid.to_bare(),
            Box::new(move |room| {
                // Convert the temporary room to its final form…
                room.to_permanent(metadata)
            }),
        ) else {
            return Err(RequestError::Generic {
                msg: "Room was modified during connection".to_string(),
            }
            .into());
        };

        let room_name = room
            .info
            .name
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or(room.info.jid.to_string());

        if save_bookmark {
            let result = self
                .bookmarks_repo
                .put(Bookmark {
                    name: room_name,
                    room_jid: room_jid.to_bare(),
                })
                .await;

            match result {
                Ok(_) => (),
                Err(error) => {
                    error!("Failed to save bookmark for room {}. {}", room_jid, error)
                }
            }
        }

        if notify_delegate {
            self.app_service
                .event_dispatcher
                .dispatch_event(ClientEvent::RoomsChanged);
        }

        Ok(self.room_factory.build(room))
    }

    async fn create_or_join_group(
        &self,
        service: &BareJid,
        user_jid: &BareJid,
        nickname: &str,
        participants: Vec<BareJid>,
        send_invites: bool,
    ) -> Result<RoomMetadata, RoomError> {
        // Load participant infos so that we can build a nice human-readable name for the group…
        let mut participant_names = vec![];
        let participants_including_self = participants
            .iter()
            .chain(iter::once(user_jid))
            .cloned()
            .collect::<Vec<_>>();

        for jid in participants_including_self.iter() {
            // TODO: First try profile from cache
            let participant_name = self
                .user_profile_service
                .load_profile(jid)
                .await?
                .and_then(|profile| profile.first_name.or(profile.nickname))
                .or(jid.node_str().map(|node| node.to_uppercase_first_letter()))
                .unwrap_or(jid.to_string());
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

        let metadata = self
            .create_or_join_room_with_config(
                service,
                user_jid,
                &group_hash,
                nickname,
                RoomConfig::group(group_name, participants_including_self.iter()),
                |room_metadata| {
                    // Try to promote all participants to owners…
                    info!("Update participant affiliations…");
                    let room_jid = room_metadata.room_jid.to_bare();
                    let room_has_been_created = room_metadata.room_has_been_created();
                    let service = self.room_management_service.clone();

                    async move {
                        let owners = participants_including_self.iter().collect::<Vec<_>>();

                        if room_has_been_created {
                            service.set_room_owners(
                                &room_jid,
                                owners.as_slice()
                            ).await
                                .context(
                                    "Failed to update user affiliations of created group to type 'owner'",
                                )
                        } else {
                            Ok(())
                        }
                    }
                },
            )
            .await?;

        // Send invites…
        if send_invites && metadata.room_has_been_created() {
            info!("Sending invites for created group…");
            let participants = participants.iter().collect::<Vec<_>>();
            self.room_participation_service
                .invite_users_to_room(&metadata.room_jid.to_bare(), participants.as_slice())
                .await?;
        }

        Ok(metadata)
    }

    async fn create_or_join_room_with_config<Fut: Future<Output = Result<()>> + 'static>(
        &self,
        service: &BareJid,
        user_jid: &BareJid,
        room_name: &str,
        nickname: &str,
        config: RoomConfig,
        perform_additional_config: impl FnOnce(&RoomMetadata) -> Fut,
    ) -> Result<RoomMetadata, RoomError> {
        let mut attempt = 0;

        // Algo is…
        // 1. Try to create or enter room with given room name and nickname
        // 2. If server returns "conflict" error (room exists and nickname is taken) append
        //    "#($ATTEMPT)" to nickname and continue at 1.
        // 2. If server returns "gone" error (room existed once but was deleted in the meantime)
        //    append "#($ATTEMPT)" to room name and continue at 1.
        // 3. Get room info
        // 4. Validate created/joined room with room info
        // 5. If validation fails and the room was created by us, delete room and return error
        // 6. Return final room jid, user and info.

        loop {
            let unique_room_name = if attempt == 0 {
                room_name.to_string()
            } else {
                format!("{}#{}", room_name, attempt)
            };
            attempt += 1;

            let room_jid =
                BareJid::from_parts(Some(&NodePart::new(&unique_room_name)?), &service.domain());
            let full_room_jid = room_jid.with_resource_str(nickname)?;

            // Insert pending room so that we don't miss any stanzas for this room while we're
            // creating (but potentially connecting to) it…
            self.insert_pending_room(&room_jid, user_jid, &nickname)?;

            // Try to create or enter the room and configure it…
            let room_config = config.clone();
            let result = self
                .room_management_service
                .create_reserved_room(
                    &full_room_jid,
                    Box::new(|form| {
                        Box::pin(async move {
                            Ok(RoomConfigResponse::Submit(
                                room_config.populate_form(&form)?,
                            ))
                        }) as PinnedFuture<_>
                    }),
                )
                .await;

            let metadata = match result {
                Ok(occupancy) => occupancy,
                Err(error) => {
                    // Remove pending room again…
                    self.connected_rooms_repo.delete(&[&room_jid]);

                    // We've received a conflict which means that the room exists but that our
                    // nickname is taken.
                    if error.is_conflict_err() {
                        // So we'll try to connect again by modifying our nickname…
                        return self
                            .join_room_by_resolving_nickname_conflict(
                                &room_jid, user_jid, nickname, None, 1,
                            )
                            .await
                            .map_err(|err| err.into());
                    }
                    // In this case the room existed in the past but was deleted. We'll modify
                    // the name and try again…
                    else if error.is_gone_err() {
                        continue;
                    }

                    return Err(error.into());
                }
            };

            let room_has_been_created = metadata.room_has_been_created();

            match config.validate(&metadata.settings) {
                Ok(_) => (),
                Err(error) => {
                    // Remove pending room again…
                    self.connected_rooms_repo.delete(&[&room_jid]);
                    // If the validation failed and we've created the room we'll destroy it again.
                    if room_has_been_created {
                        _ = self.room_management_service.destroy_room(&room_jid).await;
                    }
                    return Err(error.into());
                }
            }

            match (perform_additional_config)(&metadata).await {
                Ok(_) => (),
                Err(error) => {
                    // Remove pending room again…
                    self.connected_rooms_repo.delete(&[&room_jid]);
                    // Again, if the additional configuration fails and we've created the room
                    // we'll destroy it again.
                    if room_has_been_created {
                        _ = self.room_management_service.destroy_room(&room_jid).await;
                    }
                    return Err(error.into());
                }
            }

            return Ok(metadata);
        }
    }

    async fn join_room_by_resolving_nickname_conflict(
        &self,
        room_jid: &BareJid,
        user_jid: &BareJid,
        preferred_nickname: &str,
        password: Option<&str>,
        attempt: u32,
    ) -> Result<RoomMetadata, RoomError> {
        let mut attempt = attempt;

        loop {
            let nickname = if attempt == 0 {
                preferred_nickname.to_string()
            } else {
                format!("{}#{}", preferred_nickname, attempt)
            };
            attempt += 1;

            // Insert pending room so that we don't miss any stanzas for this room while we're
            // connecting to it…
            self.insert_pending_room(room_jid, user_jid, &nickname)?;

            info!(
                "Trying to join room {} with nickname {}…",
                room_jid, nickname
            );
            return match self
                .room_management_service
                .join_room(&room_jid.with_resource_str(&nickname)?, password)
                .await
            {
                Ok(metadata) => {
                    info!("Successfully joined room.");
                    Ok(metadata)
                }
                Err(RoomError::RequestError(error)) => {
                    // Remove pending room again…
                    self.connected_rooms_repo.delete(&[room_jid]);

                    if error.defined_condition() == Some(DefinedCondition::Conflict) {
                        info!("Nickname was already taken in room. Will try again with modified nickname…");
                        continue;
                    }

                    Err(error.into())
                }
                Err(error) => {
                    // Remove pending room again…
                    self.connected_rooms_repo.delete(&[room_jid]);
                    Err(error)
                }
            };
        }
    }

    fn insert_pending_room(
        &self,
        room_jid: &BareJid,
        user_jid: &BareJid,
        nickname: &str,
    ) -> Result<(), RoomError> {
        self.connected_rooms_repo
            .set(RoomInternals::pending(room_jid, user_jid, nickname))
            .map_err(|_| RoomError::RoomIsAlreadyConnected(room_jid.clone()))
    }
}

trait ParticipantsVecExt {
    fn group_name_hash(&self) -> String;
}

impl ParticipantsVecExt for Vec<BareJid> {
    fn group_name_hash(&self) -> String {
        let mut sorted_participant_jids =
            self.iter().map(|jid| jid.to_string()).collect::<Vec<_>>();
        sorted_participant_jids.sort();

        let mut hasher = Sha1::new();
        hasher.update(sorted_participant_jids.join(","));
        format!("{}.{:x}", GROUP_PREFIX, hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use prose_xmpp::jid;

    use super::*;

    #[test]
    fn test_group_name_for_participants() {
        assert_eq!(
            vec![
                jid!("a@prose.org").into_bare(),
                jid!("b@prose.org").into_bare(),
                jid!("c@prose.org").into_bare()
            ]
            .group_name_hash(),
            "org.prose.group.7c138d7281db96e0d42fe026a4195c85a7dc2cae".to_string()
        );

        assert_eq!(
            vec![
                jid!("a@prose.org").into_bare(),
                jid!("b@prose.org").into_bare(),
                jid!("c@prose.org").into_bare()
            ]
            .group_name_hash(),
            vec![
                jid!("c@prose.org").into_bare(),
                jid!("a@prose.org").into_bare(),
                jid!("b@prose.org").into_bare()
            ]
            .group_name_hash()
        )
    }
}
