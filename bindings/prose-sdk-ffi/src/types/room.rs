// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::{
    ClientResult, MessageResultSet, ParticipantBasicInfo, ParticipantInfo, SendMessageRequest,
};
use crate::{Emoji, Message, MessageId, RoomId};
use prose_core_client::dtos::{RoomEnvelope as CoreRoomEnvelope, RoomState as CoreRoomState};

#[derive(uniffi::Record)]
pub struct RoomEnvelope {}

#[derive(uniffi::Enum)]
pub enum RoomType {
    DirectMessage,
    Group,
    PrivateChannel,
    PublicChannel,
    Generic,
}

#[derive(uniffi::Enum)]
pub enum RoomState {
    Connecting,
    Connected,
    Disconnected {
        error: Option<String>,
        can_retry: bool,
    },
}

impl From<CoreRoomEnvelope> for RoomEnvelope {
    fn from(value: CoreRoomEnvelope) -> Self {
        match value {
            CoreRoomEnvelope::DirectMessage(room) => todo!(),
            CoreRoomEnvelope::Group(room) => todo!(),
            CoreRoomEnvelope::PrivateChannel(room) => todo!(),
            CoreRoomEnvelope::PublicChannel(room) => todo!(),
            CoreRoomEnvelope::Generic(room) => todo!(),
        }
    }
}

impl From<CoreRoomState> for RoomState {
    fn from(value: CoreRoomState) -> Self {
        match value {
            CoreRoomState::Pending | CoreRoomState::Connecting => Self::Connecting,
            CoreRoomState::Connected => Self::Connected,
            CoreRoomState::Disconnected { error, can_retry } => {
                Self::Disconnected { error, can_retry }
            }
        }
    }
}

#[uniffi::export]
#[async_trait::async_trait]
pub trait RoomBase: Send + Sync {
    fn r#type(&self) -> RoomType;
    fn state(&self) -> RoomState;
    fn id(&self) -> RoomId;
    fn name(&self) -> String;
    fn participants(&self) -> Vec<ParticipantInfo>;

    async fn send_message(&self, request: SendMessageRequest) -> ClientResult<()>;
    async fn update_message(
        &self,
        message_id: MessageId,
        request: SendMessageRequest,
    ) -> ClientResult<()>;
    async fn retract_message(&self, message_id: MessageId) -> ClientResult<()>;
    async fn toggle_reaction_to_message(
        &self,
        message_id: MessageId,
        emoji: Emoji,
    ) -> ClientResult<()>;

    async fn load_latest_messages(&self) -> ClientResult<MessageResultSet>;
    async fn load_messages_before(&self, before: MessageId) -> ClientResult<MessageResultSet>;
    async fn load_messages_with_ids(&self, ids: Vec<MessageId>) -> ClientResult<Vec<Message>>;
    async fn load_unread_messages(&self) -> ClientResult<MessageResultSet>;

    async fn set_user_is_composing(&self, is_composing: bool) -> ClientResult<()>;
    async fn load_composing_users(&self) -> ClientResult<Vec<ParticipantBasicInfo>>;

    async fn save_draft(&self, message: Option<String>) -> ClientResult<()>;
    async fn load_draft(&self) -> ClientResult<Option<String>>;

    async fn mark_as_read(&self) -> ClientResult<()>;
    async fn set_last_read_message(&self, message_id: MessageId) -> ClientResult<()>;
}
