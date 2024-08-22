// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use jid::BareJid;
use strum::IntoEnumIterator;
use xmpp_parsers::data_forms::DataForm;
use xmpp_parsers::muc::user::{Affiliation, Status};
use xmpp_parsers::stanza_error::{DefinedCondition, ErrorType, StanzaError};

use prose_xmpp::mods::muc::RoomConfigResponse;
use prose_xmpp::{mods, RequestError};

use crate::domain::general::models::Capabilities;
use crate::domain::rooms::models::{
    PublicRoomInfo, RoomAffiliation, RoomConfig, RoomError, RoomSessionInfo, RoomSessionMember,
    RoomSpec,
};
use crate::domain::rooms::services::RoomManagementService;
use crate::domain::shared::models::{MucId, OccupantId, RoomType, UserId};
use crate::dtos::Availability;
use crate::infra::xmpp::type_conversions::room_info::RoomInfo;
use crate::infra::xmpp::util::RoomOccupancyExt;
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
                id: room.jid.into_bare().into(),
                name: room.name,
            })
            .collect();
        Ok(rooms)
    }

    async fn create_or_join_room(
        &self,
        occupant_id: &OccupantId,
        room_name: &str,
        nickname: &str,
        spec: RoomSpec,
        capabilities: &Capabilities,
        availability: Availability,
    ) -> Result<RoomSessionInfo, RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();

        // Create the room…
        let occupancy = muc_mod
            .create_reserved_room(
                occupant_id.as_ref(),
                Some(nickname.to_string()),
                Some(availability.try_into()?),
                Some(capabilities.into()),
                Box::new(|form| {
                    let spec = spec.clone();
                    let room_name = room_name.to_string();

                    Box::pin(async move {
                        Ok(RoomConfigResponse::Submit(
                            spec.populate_form(&room_name, &form)?,
                        ))
                    })
                }),
            )
            .await?;

        let user_nickname = occupant_id.nickname().to_string();
        let room_jid = occupant_id.muc_id();

        let room_has_been_created = occupancy.user.status.contains(&Status::RoomHasBeenCreated);
        let room_info = self.load_room_info(&room_jid).await?;

        // Then validate it against our spec…
        if let Err(error) = spec.validate_against(&room_info) {
            // If the room was created but doesn't match our spec, we'll try to delete it again.
            if room_has_been_created {
                // Ignore the error since it would not be indicative of what happened.
                _ = muc_mod.destroy_room(&room_jid, None).await;
            }

            return Err(RoomError::RoomValidationError(error.to_string()));
        }

        let members = self.load_room_members(&room_jid).await?;
        let participants = occupancy.participants();

        Ok(RoomSessionInfo {
            room_id: room_jid.into(),
            config: RoomConfig {
                room_name: room_info.name,
                room_description: room_info.description,
                room_type: spec.room_type(),
                mam_version: room_info.features.mam_version,
                supports_self_ping_optimization: room_info.features.supports_self_ping_optimization,
            },
            topic: occupancy.subject,
            user_nickname,
            members,
            participants,
            room_has_been_created,
        })
    }

    async fn join_room(
        &self,
        occupant_id: &OccupantId,
        password: Option<&str>,
        nickname: &str,
        capabilities: &Capabilities,
        availability: Availability,
    ) -> Result<RoomSessionInfo, RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        let occupancy = muc_mod
            .enter_room(
                occupant_id.as_ref(),
                password,
                Some(nickname.to_string()),
                Some(availability.try_into()?),
                Some(capabilities.into()),
            )
            .await?;

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
                    alternate_address: None,
                },
            }
            .into());
        }

        let user_nickname = occupant_id.nickname().to_string();
        let room_jid = occupant_id.muc_id();
        let room_config = self.load_room_config(&room_jid).await?;
        let members = self.load_room_members(&room_jid).await?;
        let participants = occupancy.participants();

        Ok(RoomSessionInfo {
            room_id: room_jid.into(),
            config: room_config,
            topic: occupancy.subject,
            user_nickname,
            members,
            participants,
            room_has_been_created: false,
        })
    }

    async fn reconfigure_room(
        &self,
        room_id: &MucId,
        spec: RoomSpec,
        new_name: &str,
    ) -> Result<(), RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();

        // Reconfigure the room…
        muc_mod
            .configure_room(
                room_id,
                Box::new(|form: DataForm| {
                    let spec = spec.clone();
                    let room_name = new_name.to_string();

                    Box::pin(async move {
                        Ok(RoomConfigResponse::Submit(
                            spec.populate_form(&room_name, &form)?,
                        ))
                    })
                }),
            )
            .await?;

        let room_info = self.load_room_info(&room_id).await?;

        // Then validate it against our spec…
        if let Err(error) = spec.validate_against(&room_info) {
            return Err(RoomError::RoomValidationError(error.to_string()));
        }

        Ok(())
    }

    async fn load_room_config(&self, room_id: &MucId) -> Result<RoomConfig, RoomError> {
        let room_info = self.load_room_info(&room_id).await?;

        let room_type = 'room_type: {
            for room_spec in RoomSpec::iter() {
                if room_spec.is_satisfied_by(&room_info) {
                    break 'room_type room_spec.room_type();
                }
            }
            RoomType::Generic
        };

        Ok(RoomConfig {
            room_name: room_info.name,
            room_description: room_info.description,
            room_type,
            mam_version: room_info.features.mam_version,
            supports_self_ping_optimization: room_info.features.supports_self_ping_optimization,
        })
    }

    async fn exit_room(&self, occupant_id: &OccupantId) -> Result<(), RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        muc_mod.exit_room(occupant_id.as_ref()).await?;
        Ok(())
    }

    async fn set_room_owners<'a, 'b, 'c>(
        &'a self,
        room_id: &'b MucId,
        users: &'c [UserId],
    ) -> Result<(), RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        let owners = users
            .iter()
            .map(|user_jid| (user_jid.clone().into_inner(), Affiliation::Owner))
            .collect::<Vec<_>>();
        muc_mod.update_user_affiliations(room_id, owners).await?;
        Ok(())
    }

    async fn send_self_ping(&self, occupant_id: &OccupantId) -> Result<(), RequestError> {
        let ping_mod = self.client.get_mod::<mods::Ping>();
        ping_mod.send_ping(occupant_id.clone().into_inner()).await
    }

    async fn destroy_room(
        &self,
        room_id: &MucId,
        alternate_room: Option<MucId>,
    ) -> Result<(), RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        muc_mod
            .destroy_room(
                room_id,
                alternate_room.map(|id| id.clone().into_inner()).as_ref(),
            )
            .await?;
        Ok(())
    }
}

impl XMPPClient {
    async fn load_room_info(&self, room_id: &MucId) -> Result<RoomInfo, RoomError> {
        let caps = self.client.get_mod::<mods::Caps>();
        Ok(RoomInfo::try_from(
            caps.query_disco_info(room_id.clone(), None).await?,
        )?)
    }

    async fn load_room_members(&self, jid: &MucId) -> Result<Vec<RoomSessionMember>, RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();

        let mut members = vec![];
        let affiliations = vec![
            (Affiliation::Owner, RoomAffiliation::Owner),
            (Affiliation::Member, RoomAffiliation::Member),
            (Affiliation::Admin, RoomAffiliation::Admin),
        ];

        for (xmpp_affiliation, domain_affiliation) in affiliations {
            members.extend(
                muc_mod
                    .request_users(jid, xmpp_affiliation)
                    .await
                    .unwrap_or(vec![])
                    .into_iter()
                    .filter_map(move |user| {
                        let user_jid = user.jid.to_bare();
                        if user_jid.node().is_none() {
                            return None;
                        }
                        Some(RoomSessionMember {
                            id: UserId::from(user.jid.to_bare()),
                            affiliation: domain_affiliation.clone(),
                        })
                    }),
            )
        }

        Ok(members)
    }
}
