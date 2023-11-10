// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use jid::{BareJid, FullJid};
use xmpp_parsers::data_forms::DataForm;
use xmpp_parsers::muc::user::{Affiliation, Status};
use xmpp_parsers::stanza_error::{DefinedCondition, ErrorType, StanzaError};

use prose_wasm_utils::PinnedFuture;
use prose_xmpp::mods::muc::{RoomConfigResponse, RoomOccupancy};
use prose_xmpp::{mods, RequestError};

use crate::domain::rooms::models::{RoomError, RoomMetadata, RoomSettings};
use crate::domain::rooms::services::RoomManagementService;
use crate::dtos::PublicRoomInfo;
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
                jid: room.jid.into_bare(),
                name: room.name,
            })
            .collect();
        Ok(rooms)
    }

    async fn create_reserved_room(
        &self,
        room_jid: &FullJid,
        handler: Box<
            dyn FnOnce(DataForm) -> PinnedFuture<anyhow::Result<RoomConfigResponse>>
                + 'static
                + Send,
        >,
    ) -> Result<RoomMetadata, RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        let occupancy = muc_mod.create_reserved_room(room_jid, handler).await?;
        self.load_room_metadata(room_jid, occupancy).await
    }

    async fn join_room(
        &self,
        room_jid: &FullJid,
        password: Option<&str>,
    ) -> Result<RoomMetadata, RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        let occupancy = muc_mod.enter_room(room_jid, password).await?;

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

        self.load_room_metadata(room_jid, occupancy).await
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
    async fn load_room_metadata(
        &self,
        room_jid: &FullJid,
        occupancy: RoomOccupancy,
    ) -> Result<RoomMetadata, RoomError> {
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
}
