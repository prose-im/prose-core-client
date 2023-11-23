// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::shared::models::{RoomJid, RoomType};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait MessageMigrationDomainService: SendUnlessWasm + SyncUnlessWasm {
    async fn copy_all_messages_from_room(
        &self,
        source_room: &RoomJid,
        source_room_type: &RoomType,
        target_room: &RoomJid,
        target_room_type: &RoomType,
    ) -> Result<()>;
}
