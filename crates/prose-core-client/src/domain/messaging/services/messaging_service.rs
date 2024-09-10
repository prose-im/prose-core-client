// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::stanza::message::mam::ArchivedMessage;

use crate::domain::messaging::models::{
    Emoji, KeyTransportPayload, MessageRemoteId, MessageServerId, SendMessageRequest, ThreadId,
};
use crate::domain::shared::models::RoomId;
use crate::dtos::{MucId, UserId};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait MessagingService: SendUnlessWasm + SyncUnlessWasm {
    async fn send_message(&self, room_id: &RoomId, request: SendMessageRequest) -> Result<()>;

    async fn send_message_to_thread(
        &self,
        room_id: &RoomId,
        thread_id: &ThreadId,
        request: SendMessageRequest,
    ) -> Result<()>;

    async fn send_key_transport_message(
        &self,
        user_id: &UserId,
        message: KeyTransportPayload,
    ) -> Result<()>;

    async fn update_message(
        &self,
        room_id: &RoomId,
        message_id: &MessageRemoteId,
        body: SendMessageRequest,
    ) -> Result<()>;

    async fn retract_message(&self, room_id: &RoomId, message_id: &MessageRemoteId) -> Result<()>;

    async fn react_to_chat_message(
        &self,
        room_id: &UserId,
        message_id: &MessageRemoteId,
        emoji: &[Emoji],
    ) -> Result<()>;

    async fn react_to_muc_message(
        &self,
        room_id: &MucId,
        message_id: &MessageServerId,
        emoji: &[Emoji],
    ) -> Result<()>;

    async fn set_user_is_composing(&self, room_id: &RoomId, is_composing: bool) -> Result<()>;

    async fn send_read_receipt(&self, room_id: &RoomId, message_id: &MessageRemoteId)
        -> Result<()>;

    async fn relay_archived_message_to_room(
        &self,
        room_id: &RoomId,
        message: ArchivedMessage,
    ) -> Result<()>;
}
