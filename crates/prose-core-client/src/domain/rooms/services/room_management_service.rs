// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use jid::{BareJid, FullJid};
use xmpp_parsers::data_forms::DataForm;

use prose_wasm_utils::{PinnedFuture, SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::mods;
use prose_xmpp::mods::muc::RoomConfigResponse;

use crate::domain::rooms::models::{RoomError, RoomMetadata};
use crate::domain::rooms::services::RoomParticipationService;

type ConfigureRoomHandler =
    Box<dyn FnOnce(DataForm) -> PinnedFuture<anyhow::Result<RoomConfigResponse>> + 'static + Send>;

#[async_trait]
pub trait RoomManagementService:
    RoomParticipationService + SendUnlessWasm + SyncUnlessWasm
{
    async fn load_public_rooms(
        &self,
        muc_service: &BareJid,
    ) -> Result<Vec<mods::muc::Room>, RoomError>;

    async fn create_reserved_room(
        &self,
        room_jid: &FullJid,
        handler: ConfigureRoomHandler,
    ) -> Result<RoomMetadata, RoomError>;

    async fn join_room(
        &self,
        room_jid: &FullJid,
        password: Option<&str>,
    ) -> Result<RoomMetadata, RoomError>;

    async fn set_room_owners(
        &self,
        room_jid: &BareJid,
        users: &[&BareJid],
    ) -> Result<(), RoomError>;

    async fn destroy_room(&self, room_jid: &BareJid) -> Result<(), RoomError>;
}

#[cfg(feature = "test")]
mockall::mock! {
    pub RoomManagementService {}

    #[async_trait]
    impl RoomManagementService for RoomManagementService {
        async fn load_public_rooms(
            &self,
            muc_service: &BareJid,
        ) -> Result<Vec<mods::muc::Room>, RoomError>;

        async fn create_reserved_room(
            &self,
            room_jid: &FullJid,
            handler: ConfigureRoomHandler,
        ) -> Result<RoomMetadata, RoomError>;

        async fn join_room<'a, 'b, 'c>(
            &'a self,
            room_jid: &'b FullJid,
            password: Option<&'c str>,
        ) -> Result<RoomMetadata, RoomError>;

        async fn set_room_owners<'a, 'b, 'c, 'd>(
            &'a self,
            room_jid: &'b BareJid,
            users: &'c [&'d BareJid],
        ) -> Result<(), RoomError>;

        async fn destroy_room(&self, room_jid: &BareJid) -> Result<(), RoomError>;
    }

    #[async_trait]
    impl RoomParticipationService for RoomManagementService {
        async fn invite_users_to_room<'a, 'b, 'c, 'd>(
            &'a self,
            room_jid: &'b BareJid,
            participants: &'c [&'d BareJid],
        ) -> Result<(), RoomError>;
    }
}
