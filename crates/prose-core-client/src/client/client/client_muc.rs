// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::iter;

use anyhow::{bail, Result};
use jid::{BareJid, FullJid, Jid};
use sha1::{Digest, Sha1};
use tracing::{error, info};
use xmpp_parsers::bookmarks2::{Autojoin, Conference};
use xmpp_parsers::muc::user::Affiliation;

use prose_xmpp::stanza::muc::{mediated_invite, DirectInvite, MediatedInvite};
use prose_xmpp::stanza::ConferenceBookmark;
use prose_xmpp::{mods, RequestError};

use crate::avatar_cache::AvatarCache;
use crate::client::room::RoomEnvelope;
use crate::data_cache::DataCache;
use crate::types::muc::{RoomConfig, RoomMetadata, RoomSettings};
use crate::types::{muc, ConnectedRoom};
use crate::util::StringExt;
use crate::ClientEvent;

use super::Client;

#[derive(thiserror::Error, Debug, PartialEq)]
enum MUCError {
    #[error("Server does not support MUC (XEP-0045)")]
    Unsupported,
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub fn connected_rooms(&self) -> Vec<ConnectedRoom<D, A>> {
        self.inner
            .connected_rooms
            .read()
            .values()
            .filter_map(|envelope| ConnectedRoom::try_from(envelope.clone()).ok())
            .collect()
    }

    pub async fn load_public_rooms(&self) -> Result<Vec<mods::muc::Room>> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        muc_mod.load_public_rooms(&self.muc_service()?.jid).await
    }

    pub async fn join_room_with_jid(
        &self,
        room_jid: &BareJid,
        password: Option<&str>,
    ) -> Result<ConnectedRoom<D, A>, RequestError> {
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

    pub async fn create_direct_message(
        &self,
        participants: &[BareJid],
    ) -> Result<ConnectedRoom<D, A>> {
        if participants.is_empty() {
            bail!("Group must have at least one other participant.")
        }

        let room = self
            .create_or_join_room(CreateOrEnterRoomRequest {
                r#type: CreateOrEnterRoomRequestType::Create {
                    service: self.muc_service()?.jid,
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

    pub async fn create_private_channel(
        &self,
        channel_name: impl AsRef<str>,
    ) -> Result<ConnectedRoom<D, A>> {
        // Create room…
        info!(
            "Creating private channel with name {}…",
            channel_name.as_ref()
        );

        let room = self
            .create_or_join_room(CreateOrEnterRoomRequest {
                r#type: CreateOrEnterRoomRequestType::Create {
                    service: self.muc_service()?.jid,
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

    pub async fn create_public_channel(
        &self,
        channel_name: impl AsRef<str>,
    ) -> Result<ConnectedRoom<D, A>> {
        // Create room…
        info!(
            "Creating public channel with name {}…",
            channel_name.as_ref()
        );

        let room = self
            .create_or_join_room(CreateOrEnterRoomRequest {
                r#type: CreateOrEnterRoomRequestType::Create {
                    service: self.muc_service()?.jid,
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
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub(super) async fn load_bookmarks(&self) -> Result<HashMap<BareJid, ConferenceBookmark>> {
        let bookmark_mod = self.client.get_mod::<mods::Bookmark>();
        Ok(bookmark_mod
            .load_bookmarks()
            .await?
            .into_iter()
            .map(|bookmark| (bookmark.jid.to_bare(), bookmark))
            .collect())
    }

    pub(super) async fn handle_direct_invite(&self, from: Jid, invite: DirectInvite) -> Result<()> {
        info!("Joining room {} after receiving direct invite…", from);

        self.create_or_join_room(CreateOrEnterRoomRequest {
            r#type: CreateOrEnterRoomRequestType::Join {
                room_jid: invite.jid,
                nickname: None,
                password: invite.password,
            },
            save_bookmark: true,
            notify_delegate: true,
        })
        .await?;

        Ok(())
    }

    pub(super) async fn handle_mediated_invite(
        &self,
        from: Jid,
        invite: MediatedInvite,
    ) -> Result<()> {
        info!("Joining room {} after receiving mediated invite…", from);

        self.create_or_join_room(CreateOrEnterRoomRequest {
            r#type: CreateOrEnterRoomRequestType::Join {
                room_jid: from.to_bare(),
                nickname: None,
                password: invite.password,
            },
            save_bookmark: true,
            notify_delegate: true,
        })
        .await?;

        Ok(())
    }

    pub(super) async fn handle_changed_bookmarks(
        &self,
        _bookmarks: Vec<ConferenceBookmark>,
    ) -> Result<()> {
        Ok(())
    }

    pub(super) async fn handle_published_bookmarks2(
        &self,
        _bookmarks: Vec<ConferenceBookmark>,
    ) -> Result<()> {
        Ok(())
    }

    pub(super) async fn handle_retracted_bookmarks2(&self, _jids: Vec<Jid>) -> Result<()> {
        Ok(())
    }

    pub(super) async fn enter_room(
        &self,
        room_jid: &BareJid,
        nickname: Option<&str>,
        password: Option<&str>,
    ) -> Result<(), RequestError> {
        self.create_or_join_room(CreateOrEnterRoomRequest {
            r#type: CreateOrEnterRoomRequestType::Join {
                room_jid: room_jid.clone(),
                nickname: nickname.map(ToString::to_string),
                password: password.map(ToString::to_string),
            },
            save_bookmark: false,
            notify_delegate: false,
        })
        .await?;
        Ok(())
    }

    pub(super) async fn remove_and_publish_bookmarks(&self, jids: &[BareJid]) -> Result<()> {
        if jids.is_empty() {
            return Ok(());
        }

        info!("Deleting {} bookmarks…", jids.len());
        let mut bookmarks = self.inner.bookmarks.write();
        let bookmarks_to_delete = jids.iter().collect::<HashSet<_>>();
        let bookmarks_len = bookmarks.len();

        bookmarks.retain(|jid, _| !bookmarks_to_delete.contains(jid));

        if bookmarks.len() == bookmarks_len {
            return Ok(());
        }

        let bookmarks_to_publish = bookmarks.values().cloned().collect::<Vec<_>>();
        drop(bookmarks);

        info!("Publishing bookmarks…");
        let bookmark_mod = self.client.get_mod::<mods::Bookmark>();
        bookmark_mod.publish_bookmarks(bookmarks_to_publish).await?;
        Ok(())
    }
}

#[cfg(feature = "debug")]
impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub async fn load_bookmarks_dbg(&self) -> Result<Vec<ConferenceBookmark>> {
        Ok(self.load_bookmarks().await?.values().cloned().collect())
    }

    pub async fn destroy_room(&self, room_jid: &BareJid) -> Result<()> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        muc_mod.destroy_room(room_jid).await?;
        Ok(())
    }

    pub async fn delete_bookmark(&self, jid: &Jid) -> Result<()> {
        self.remove_and_publish_bookmarks(&[jid.to_bare()]).await
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    fn muc_service(&self) -> Result<muc::Service, MUCError> {
        let Some(service) = self.inner.muc_service.read().clone() else {
            return Err(MUCError::Unsupported);
        };
        return Ok(service);
    }

    async fn insert_and_publish_bookmark(&self, bookmark: ConferenceBookmark) -> Result<()> {
        let bare_jid = bookmark.jid.to_bare();
        info!("Inserting bookmark {}…", bare_jid);

        let bookmarks_to_publish = {
            let mut bookmarks = self.inner.bookmarks.write();

            if bookmarks.get(&bare_jid) == Some(&bookmark) {
                return Ok(());
            }

            bookmarks.insert(bare_jid, bookmark);
            bookmarks.values().cloned().collect::<Vec<_>>()
        };

        info!("Publishing bookmarks…");
        let bookmark_mod = self.client.get_mod::<mods::Bookmark>();
        bookmark_mod.publish_bookmarks(bookmarks_to_publish).await?;
        Ok(())
    }
}

enum CreateRoomType {
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

enum CreateOrEnterRoomRequestType {
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

struct CreateOrEnterRoomRequest {
    r#type: CreateOrEnterRoomRequestType,
    save_bookmark: bool,
    notify_delegate: bool,
}

mod room_handling {
    #[derive(thiserror::Error, Debug)]
    pub(super) enum Error {
        #[error("Room is already connected ({0}).")]
        RoomIsAlreadyConnected(BareJid),
        #[error(transparent)]
        RequestError(#[from] RequestError),
        #[error(transparent)]
        RoomValidationError(#[from] RoomValidationError),
        #[error(transparent)]
        Anyhow(#[from] anyhow::Error),
        #[error(transparent)]
        JidError(#[from] jid::Error),
        #[error(transparent)]
        ParseError(#[from] prose_xmpp::ParseError),
    }

    use anyhow::Context;
    use jid::NodePart;
    use xmpp_parsers::muc::user::Status;
    use xmpp_parsers::stanza_error::{DefinedCondition, ErrorType, StanzaError};

    use crate::types::muc::RoomValidationError;
    use prose_xmpp::mods::muc::{RoomConfigResponse, RoomOccupancy};

    use super::*;

    pub(super) const GROUP_PREFIX: &str = "org.prose.group";
    pub(super) const PRIVATE_CHANNEL_PREFIX: &str = "org.prose.private-channel";
    pub(super) const PUBLIC_CHANNEL_PREFIX: &str = "org.prose.public-channel";

    impl<D: DataCache, A: AvatarCache> Client<D, A> {
        pub(super) async fn create_or_join_room(
            &self,
            CreateOrEnterRoomRequest {
                r#type,
                save_bookmark,
                notify_delegate,
            }: CreateOrEnterRoomRequest,
        ) -> Result<ConnectedRoom<D, A>, Error> {
            let user_jid = self
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
                        let channel_id = self.inner.id_provider.new_id();

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
                Err(Error::RoomIsAlreadyConnected(room_jid)) => {
                    if let Some(room) = self
                        .inner
                        .connected_rooms
                        .read()
                        .get(&room_jid)
                        .and_then(|room| ConnectedRoom::try_from(room.clone()).ok())
                    {
                        return Ok(room);
                    };
                    return Err(Error::RoomIsAlreadyConnected(room_jid));
                }
                Err(error) => return Err(error),
            };

            // It could be the case that the room_jid was modified, i.e. if the preferred JID was
            // taken already.
            let room_jid = metadata.room_jid.clone();

            let room = {
                let mut connected_rooms = self.inner.connected_rooms.write();
                let Some(room) = connected_rooms.remove(&room_jid.to_bare()) else {
                    return Err(RequestError::Generic {
                        msg: "Room was modified during connection".to_string(),
                    }
                    .into());
                };

                // Convert the temporary room to its final form…
                let room = room.promote_to_permanent_room(metadata).map_err(|err| {
                    RequestError::Generic {
                        msg: err.to_string(),
                    }
                })?;

                connected_rooms.insert(room_jid.to_bare(), room.clone());
                room
            };

            let room_name = room.name().map(ToString::to_string);

            if save_bookmark {
                let bookmark = ConferenceBookmark {
                    jid: room_jid.to_bare().into(),
                    conference: Conference {
                        autojoin: Autojoin::True,
                        name: room_name,
                        // We're not saving a nickname so that we keep using the node of the
                        // logged-in user's JID instead.
                        nick: None,
                        password: None,
                        extensions: vec![],
                    },
                };
                match self.insert_and_publish_bookmark(bookmark).await {
                    Ok(_) => (),
                    Err(error) => {
                        error!("Failed to save bookmark for room {}. {}", room_jid, error)
                    }
                }
            }

            if notify_delegate {
                self.send_event(ClientEvent::RoomsChanged)
            }

            Ok(
                ConnectedRoom::try_from(room).map_err(|err| RequestError::Generic {
                    msg: err.to_string(),
                })?,
            )
        }

        async fn join_room_by_resolving_nickname_conflict(
            &self,
            room_jid: &BareJid,
            user_jid: &BareJid,
            preferred_nickname: &str,
            password: Option<&str>,
            attempt: u32,
        ) -> Result<RoomMetadata, Error> {
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
                    .join_room(&room_jid.with_resource_str(&nickname)?, password)
                    .await
                {
                    Ok(metadata) => {
                        info!("Successfully joined room.");
                        Ok(metadata)
                    }
                    Err(Error::RequestError(error)) => {
                        // Remove pending room again…
                        self.inner.connected_rooms.write().remove(&room_jid);

                        if error.defined_condition() == Some(DefinedCondition::Conflict) {
                            info!("Nickname was already taken in room. Will try again with modified nickname…");
                            continue;
                        }

                        Err(error.into())
                    }
                    Err(error) => {
                        // Remove pending room again…
                        self.inner.connected_rooms.write().remove(&room_jid);
                        Err(error)
                    }
                };
            }
        }

        async fn join_room(
            &self,
            room_jid: &FullJid,
            password: Option<&str>,
        ) -> Result<RoomMetadata, Error> {
            let muc_mod = self.client.get_mod::<mods::MUC>();
            let occupancy = muc_mod.enter_room(&room_jid, password.as_deref()).await?;

            // If we accidentally created the room, we'll return an ItemNotFound error since our
            // actual intention was to join an existing room.
            if occupancy.user.status.contains(&Status::RoomHasBeenCreated) {
                return Err(RequestError::XMPP {
                    err: StanzaError {
                        type_: ErrorType::Cancel,
                        by: None,
                        defined_condition: DefinedCondition::ItemNotFound,
                        texts: Default::default(),
                        other: None,
                    },
                }
                .into());
            }

            return self.gather_metadata_for_room(room_jid, occupancy).await;
        }

        async fn gather_metadata_for_room(
            &self,
            room_jid: &FullJid,
            occupancy: RoomOccupancy,
        ) -> Result<RoomMetadata, Error> {
            let caps = self.client.get_mod::<mods::Caps>();
            let settings =
                RoomSettings::try_from(caps.query_disco_info(room_jid.to_bare(), None).await?)?;

            // When creating a group we change all "members" to "owners", so at least for Prose groups
            // this should work as expected. In case it fails we ignore the error, which can happen
            // for channels.
            let muc_mod = self.client.get_mod::<mods::MUC>();
            let members = muc_mod
                .request_users(&room_jid.to_bare(), Affiliation::Owner)
                .await
                .unwrap_or(vec![])
                .into_iter()
                .map(|user| user.jid.to_bare())
                .collect::<Vec<_>>();

            Ok(RoomMetadata {
                room_jid: room_jid.clone(),
                occupancy,
                settings,
                members,
            })
        }

        async fn create_or_join_group(
            &self,
            service: &BareJid,
            user_jid: &BareJid,
            nickname: &str,
            participants: Vec<BareJid>,
            send_invites: bool,
        ) -> Result<RoomMetadata, Error> {
            // Load participant infos so that we can build a nice human-readable name for the group…
            let mut participant_names = vec![];
            let participants_including_self = participants
                .iter()
                .chain(iter::once(user_jid))
                .cloned()
                .collect::<Vec<_>>();

            for jid in participants_including_self.iter() {
                let participant_name = self
                    .load_user_profile(jid.clone(), Default::default())
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
                        let muc_mod = self.client.get_mod::<mods::MUC>();
                        let room_jid = room_metadata.room_jid.to_bare();
                        let room_has_been_created = room_metadata.room_has_been_created();
                        let owners = participants_including_self
                            .iter()
                            .map(|jid| (jid.clone(), Affiliation::Owner))
                            .collect::<Vec<_>>();

                    async move {
                        if room_has_been_created {
                            muc_mod
                                .update_user_affiliations(&room_jid, owners)
                                .await
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
                let muc_mod = self.client.get_mod::<mods::MUC>();
                muc_mod
                    .send_mediated_invite(
                        &metadata.room_jid.to_bare(),
                        MediatedInvite {
                            invites: participants
                                .into_iter()
                                .map(|participant| mediated_invite::Invite {
                                    from: None,
                                    to: Some(participant.into()),
                                    reason: None,
                                })
                                .collect(),
                            password: None,
                        },
                    )
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
        ) -> Result<RoomMetadata, Error> {
            let muc_mod = self.client.get_mod::<mods::MUC>();
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

                let room_jid = BareJid::from_parts(
                    Some(&NodePart::new(&unique_room_name)?),
                    &service.domain(),
                );
                let full_room_jid = room_jid.with_resource_str(nickname)?;

                // Insert pending room so that we don't miss any stanzas for this room while we're
                // creating (but potentially connecting to) it…
                self.insert_pending_room(&room_jid, user_jid, &nickname)?;

                // Try to create or enter the room and configure it…
                let room_config = config.clone();
                let result = muc_mod
                    .create_reserved_room(&full_room_jid, |form| async move {
                        Ok(RoomConfigResponse::Submit(
                            room_config.populate_form(&form)?,
                        ))
                    })
                    .await;

                let occupancy = match result {
                    Ok(occupancy) => occupancy,
                    Err(error) => {
                        // Remove pending room again…
                        self.inner.connected_rooms.write().remove(&room_jid);

                        // We've received a conflict which means that the room exists but that our
                        // nickname is taken.
                        if error.defined_condition() == Some(DefinedCondition::Conflict) {
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
                        else if error.defined_condition() == Some(DefinedCondition::Gone) {
                            continue;
                        }

                        return Err(error.into());
                    }
                };

                let room_has_been_created =
                    occupancy.user.status.contains(&Status::RoomHasBeenCreated);

                let metadata = match self
                    .gather_metadata_for_room(&full_room_jid, occupancy)
                    .await
                {
                    Ok(metadata) => metadata,
                    Err(error) => {
                        // Remove pending room again…
                        self.inner.connected_rooms.write().remove(&room_jid);
                        if room_has_been_created {
                            _ = muc_mod.destroy_room(&room_jid).await;
                        }
                        return Err(error.into());
                    }
                };

                match config.validate(&metadata.settings) {
                    Ok(_) => (),
                    Err(error) => {
                        // Remove pending room again…
                        self.inner.connected_rooms.write().remove(&room_jid);
                        // If the validation failed and we've created the room we'll destroy it again.
                        if room_has_been_created {
                            _ = muc_mod.destroy_room(&room_jid).await;
                        }
                        return Err(error.into());
                    }
                }

                match (perform_additional_config)(&metadata).await {
                    Ok(_) => (),
                    Err(error) => {
                        // Remove pending room again…
                        self.inner.connected_rooms.write().remove(&room_jid);
                        // Again, if the additional configuration fails and we've created the room
                        // we'll destroy it again.
                        if room_has_been_created {
                            _ = muc_mod.destroy_room(&metadata.room_jid.to_bare()).await;
                        }
                        return Err(error.into());
                    }
                }

                return Ok(metadata);
            }
        }

        fn insert_pending_room(
            &self,
            room_jid: &BareJid,
            user_jid: &BareJid,
            nickname: &str,
        ) -> Result<(), Error> {
            let mut connected_rooms = self.inner.connected_rooms.write();

            if connected_rooms.contains_key(&room_jid) {
                return Err(Error::RoomIsAlreadyConnected(room_jid.clone()));
            }

            connected_rooms.insert(
                room_jid.clone(),
                RoomEnvelope::pending(room_jid, user_jid, nickname, self),
            );

            Ok(())
        }
    }

    impl From<Error> for RequestError {
        fn from(value: Error) -> Self {
            if let Error::RequestError(error) = value {
                return error;
            }
            RequestError::Generic {
                msg: value.to_string(),
            }
        }
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
        format!("{}.{:x}", room_handling::GROUP_PREFIX, hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prose_xmpp::jid_str;

    #[test]
    fn test_group_name_for_participants() {
        assert_eq!(
            vec![
                jid_str!("a@prose.org").into_bare(),
                jid_str!("b@prose.org").into_bare(),
                jid_str!("c@prose.org").into_bare()
            ]
            .group_name_hash(),
            "org.prose.group.7c138d7281db96e0d42fe026a4195c85a7dc2cae".to_string()
        );

        assert_eq!(
            vec![
                jid_str!("a@prose.org").into_bare(),
                jid_str!("b@prose.org").into_bare(),
                jid_str!("c@prose.org").into_bare()
            ]
            .group_name_hash(),
            vec![
                jid_str!("c@prose.org").into_bare(),
                jid_str!("a@prose.org").into_bare(),
                jid_str!("b@prose.org").into_bare()
            ]
            .group_name_hash()
        )
    }
}
