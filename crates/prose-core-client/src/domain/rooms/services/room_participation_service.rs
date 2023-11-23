// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use jid::BareJid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::rooms::models::RoomError;
use crate::dtos::RoomJid;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait RoomParticipationService: SendUnlessWasm + SyncUnlessWasm {
    async fn invite_users_to_room(
        &self,
        room_jid: &RoomJid,
        participants: &[BareJid],
    ) -> Result<(), RoomError>;

    async fn grant_membership(
        &self,
        room_jid: &RoomJid,
        participant: &BareJid,
    ) -> Result<(), RoomError>;
}
