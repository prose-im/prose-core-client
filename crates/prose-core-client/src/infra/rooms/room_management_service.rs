// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use jid::{BareJid, FullJid};
use strum::IntoEnumIterator;
use xmpp_parsers::muc::user::{Affiliation, Status};
use xmpp_parsers::stanza_error::{DefinedCondition, ErrorType, StanzaError};

use prose_wasm_utils::PinnedFuture;
use prose_xmpp::mods::muc::RoomConfigResponse;
use prose_xmpp::{mods, RequestError};

use crate::domain::rooms::models::{RoomError, RoomSessionInfo, RoomSpec};
use crate::domain::rooms::services::RoomManagementService;
use crate::domain::shared::models::RoomType;
use crate::dtos::{PublicRoomInfo, RoomJid};
use crate::infra::xmpp::type_conversions::room_info::RoomInfo;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl RoomManagementService for XMPPClient {
    async fn load_public_rooms(
        &self,
        muc_service: &BareJid,
    ) -> Result<Vec<PublicRoomInfo>, RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        let rooms = muc_mod
            .load_public_rooms(muc_service)
            .await?
            .into_iter()
            .map(|room| PublicRoomInfo {
                jid: room.jid.into_bare().into(),
                name: room.name,
            })
            .collect();
        Ok(rooms)
    }

    async fn create_or_join_room(
        &self,
        room_jid: &FullJid,
        room_name: &str,
        spec: RoomSpec,
    ) -> Result<RoomSessionInfo, RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();

        // Create the room…
        let occupancy = muc_mod
            .create_reserved_room(
                &room_jid,
                Box::new(|form| {
                    let spec = spec.clone();
                    let room_name = room_name.to_string();

                    Box::pin(async move {
                        Ok(RoomConfigResponse::Submit(
                            spec.populate_form(&room_name, &form)?,
                        ))
                    }) as PinnedFuture<_>
                }),
            )
            .await?;

        let user_nickname = room_jid.resource_str().to_string();
        let room_jid = RoomJid::from(room_jid.to_bare());

        let room_has_been_created = occupancy.user.status.contains(&Status::RoomHasBeenCreated);
        let room_info = self.load_room_info(&room_jid).await?;

        // Then validate it against our spec…
        if let Err(error) = spec.validate_against(&room_info) {
            // If the room was created but doesn't match our spec, we'll try to delete it again.
            if room_has_been_created {
                // Ignore the error since it would not be indicative of what happened.
                _ = muc_mod.destroy_room(&room_jid).await;
            }

            return Err(RoomError::RoomValidationError(error.to_string()));
        }

        let members = self.load_room_owners(&room_jid).await?;

        Ok(RoomSessionInfo {
            room_jid,
            room_name: room_info.name,
            room_description: room_info.description,
            room_type: spec.room_type(),
            user_nickname,
            members,
            room_has_been_created,
        })
    }

    async fn join_room(
        &self,
        room_jid: &FullJid,
        password: Option<&str>,
    ) -> Result<RoomSessionInfo, RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        let occupancy = muc_mod.enter_room(&room_jid, password).await?;

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

        let user_nickname = room_jid.resource_str().to_string();
        let room_jid = RoomJid::from(room_jid.to_bare());
        let room_info = self.load_room_info(&room_jid).await?;

        let room_type = 'room_type: {
            for room_spec in RoomSpec::iter() {
                if room_spec.is_satisfied_by(&room_info) {
                    break 'room_type room_spec.room_type();
                }
            }
            RoomType::Generic
        };

        let members = self.load_room_owners(&room_jid).await?;

        Ok(RoomSessionInfo {
            room_jid,
            room_name: room_info.name,
            room_description: room_info.description,
            room_type,
            user_nickname,
            members,
            room_has_been_created: false,
        })
    }

    async fn exit_room(&self, room_jid: &FullJid) -> Result<(), RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        muc_mod.exit_room(room_jid).await?;
        Ok(())
    }

    async fn set_room_owners<'a, 'b, 'c>(
        &'a self,
        room_jid: &'b BareJid,
        users: &'c [&BareJid],
    ) -> Result<(), RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        let owners = users
            .iter()
            .map(|user_jid| ((*user_jid).clone(), Affiliation::Owner))
            .collect::<Vec<_>>();
        muc_mod.update_user_affiliations(room_jid, owners).await?;
        Ok(())
    }

    async fn destroy_room(&self, room_jid: &BareJid) -> Result<(), RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        muc_mod.destroy_room(room_jid).await?;
        Ok(())
    }
}

impl XMPPClient {
    async fn load_room_info(&self, jid: &RoomJid) -> Result<RoomInfo, RoomError> {
        let caps = self.client.get_mod::<mods::Caps>();
        Ok(RoomInfo::try_from(
            caps.query_disco_info(jid.clone().into_inner(), None)
                .await?,
        )?)
    }

    async fn load_room_owners(&self, jid: &RoomJid) -> Result<Vec<BareJid>, RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        // When creating a group we change all "members" to "owners", so at least for Prose groups
        // this should work as expected. In case it fails we ignore the error, which can happen
        // for channels.
        Ok(muc_mod
            .request_users(jid, Affiliation::Owner)
            .await
            .unwrap_or(vec![])
            .into_iter()
            .map(|user| user.jid.to_bare())
            .collect::<Vec<_>>())
    }
}
