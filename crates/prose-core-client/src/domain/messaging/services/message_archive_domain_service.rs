// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::encryption::models::DecryptionContext;
use anyhow::Result;
use async_trait::async_trait;
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::rooms::models::Room;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait MessageArchiveDomainService: SendUnlessWasm + SyncUnlessWasm {
    /// Returns `true` if new messages were found.
    async fn catchup_room(&self, room: &Room, context: DecryptionContext) -> Result<bool>;
}
