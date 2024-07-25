// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::messaging::models::{
    ArchivedMessageRef, MessageId, MessageLike, MessageRemoteId, MessageServerId, MessageTargetId,
};
use crate::domain::shared::models::{AccountId, RoomId};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait MessagesRepository: SendUnlessWasm + SyncUnlessWasm {
    /// Returns all parts (MessageLike) that make up message with `id`. Sorted chronologically.
    async fn get(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        id: &MessageId,
    ) -> Result<Vec<MessageLike>>;
    /// Returns all parts (MessageLike) that make up all messages in `ids`. Sorted chronologically.
    async fn get_all(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        ids: &[MessageId],
    ) -> Result<Vec<MessageLike>>;
    /// Returns all messages that target any IDs contained in `targeted_id` and are newer
    /// than `newer_than`.
    async fn get_messages_targeting(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        targeted_ids: &[MessageTargetId],
        newer_than: &DateTime<Utc>,
    ) -> Result<Vec<MessageLike>>;
    async fn contains(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        id: &MessageServerId,
    ) -> Result<bool>;
    async fn append(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        messages: &[MessageLike],
    ) -> Result<()>;
    async fn clear_cache(&self, account: &AccountId) -> Result<()>;

    async fn resolve_server_id_to_message_id(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        server_id: &MessageServerId,
    ) -> Result<Option<MessageId>>;

    async fn resolve_remote_id_to_message_id(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        remote_id: &MessageRemoteId,
    ) -> Result<Option<MessageId>>;

    async fn resolve_message_id_to_remote_id(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        id: &MessageId,
    ) -> Result<Option<MessageRemoteId>>;

    /// Returns the latest message, if available, that has a `stanza_id` set and was received
    /// before `before` (if set).
    async fn get_last_received_message(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        before: Option<DateTime<Utc>>,
    ) -> Result<Option<ArchivedMessageRef>>;

    /// Returns all messages with a timestamp greater than `after`.
    async fn get_messages_after(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        after: DateTime<Utc>,
    ) -> Result<Vec<MessageLike>>;
}
