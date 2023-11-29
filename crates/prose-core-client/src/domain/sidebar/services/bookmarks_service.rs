// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::sidebar::models::Bookmark;
use crate::dtos::RoomId;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait BookmarksService: SendUnlessWasm + SyncUnlessWasm {
    async fn load_bookmarks(&self) -> Result<Vec<Bookmark>>;
    async fn save_bookmark(&self, bookmark: &Bookmark) -> Result<()>;
    async fn delete_bookmark(&self, jid: &RoomId) -> Result<()>;
}
