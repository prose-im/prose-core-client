// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait DraftsRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn get(&self, room_id: &BareJid) -> Result<Option<String>>;
    async fn set(&self, room_id: &BareJid, draft: Option<&str>) -> Result<()>;
}
