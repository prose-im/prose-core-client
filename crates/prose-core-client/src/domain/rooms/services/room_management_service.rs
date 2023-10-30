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

type ConfigureRoomHandler =
    Box<dyn FnOnce(DataForm) -> PinnedFuture<anyhow::Result<RoomConfigResponse>> + 'static + Send>;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait RoomManagementService: SendUnlessWasm + SyncUnlessWasm {
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
