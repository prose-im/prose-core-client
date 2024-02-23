// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::messaging::models::{MessageId, MessageLike};
use crate::domain::shared::models::RoomId;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait MessagesRepository: SendUnlessWasm + SyncUnlessWasm {
    /// Returns all parts (MessageLike) that make up message with `id`. Sorted chronologically.
    async fn get(&self, room_id: &RoomId, id: &MessageId) -> Result<Vec<MessageLike>>;
    /// Returns all parts (MessageLike) that make up all messages in `ids`. Sorted chronologically.
    async fn get_all(&self, room_id: &RoomId, ids: &[MessageId]) -> Result<Vec<MessageLike>>;
    /// Returns all messages that target any IDs contained in `targeted_ids` and are newer
    /// than `newer_than`.
    async fn get_messages_targeting(
        &self,
        room_id: &RoomId,
        targeted_ids: &[MessageId],
        newer_than: &DateTime<Utc>,
    ) -> Result<Vec<MessageLike>>;
    async fn contains(&self, id: &MessageId) -> Result<bool>;
    async fn append(&self, room_id: &RoomId, messages: &[MessageLike]) -> Result<()>;
    async fn clear_cache(&self) -> Result<()>;
}
