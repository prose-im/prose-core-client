// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::rooms::models::Bookmark;

#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait BookmarksRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn get_all(&self) -> Result<Vec<Bookmark>>;
    async fn put(&self, bookmark: Bookmark) -> Result<()>;
    async fn delete(&self, room_jids: &[BareJid]) -> Result<()>;
}
