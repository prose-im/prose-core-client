// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::{
    ClientResult, MessageResultSet, ParticipantBasicInfo, ParticipantInfo, SendMessageRequest,
};
use crate::{Emoji, Message, MessageId, RoomId, UserId};
use prose_core_client::dtos::{
    MessageId as CoreMessageId, RoomEnvelope as CoreRoomEnvelope, RoomState as CoreRoomState,
    UserId as CoreUserId,
};
use prose_core_client::services::{
    DirectMessage, Generic, Group, PrivateChannel, PublicChannel, Room as CoreRoom,
};
use std::sync::Arc;

#[derive(uniffi::Enum)]
pub enum RoomEnvelope {
    DirectMessage(Arc<RoomDirectMessage>),
    Group(Arc<RoomGroup>),
    PrivateChannel(Arc<RoomPrivateChannel>),
    PublicChannel(Arc<RoomPublicChannel>),
    Generic(Arc<RoomGeneric>),
}

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

#[derive(uniffi::Object)]
pub struct RoomDirectMessage {
    room: CoreRoom<DirectMessage>,
}

#[derive(uniffi::Object)]
pub struct RoomGroup {
    room: CoreRoom<Group>,
}

#[derive(uniffi::Object)]
pub struct RoomPrivateChannel {
    room: CoreRoom<PrivateChannel>,
}

#[derive(uniffi::Object)]
pub struct RoomPublicChannel {
    room: CoreRoom<PublicChannel>,
}

#[derive(uniffi::Object)]
pub struct RoomGeneric {
    room: CoreRoom<Generic>,
}

macro_rules! base_room_impl {
    ($t:ident) => {
        #[uniffi::export(async_runtime = "tokio")]
        #[async_trait::async_trait]
        impl RoomBase for $t {
            fn state(&self) -> RoomState {
                self.room.state().into()
            }

            fn id(&self) -> RoomId {
                self.room.jid().clone().into()
            }

            fn name(&self) -> String {
                self.room
                    .name()
                    .as_deref()
                    .unwrap_or("<untitled>")
                    .to_string()
            }

            fn participants(&self) -> Vec<ParticipantInfo> {
                self.room
                    .participants()
                    .into_iter()
                    .map(Into::into)
                    .collect()
            }

            async fn send_message(&self, request: SendMessageRequest) -> ClientResult<()> {
                self.room.send_message(request.into()).await?;
                Ok(())
            }

            async fn update_message(
                &self,
                message_id: MessageId,
                request: SendMessageRequest,
            ) -> ClientResult<()> {
                self.room
                    .update_message(message_id.into(), request.into())
                    .await?;
                Ok(())
            }

            async fn retract_message(&self, message_id: MessageId) -> ClientResult<()> {
                self.room.retract_message(message_id.into()).await?;
                Ok(())
            }

            async fn toggle_reaction_to_message(
                &self,
                message_id: MessageId,
                emoji: Emoji,
            ) -> ClientResult<()> {
                self.room
                    .toggle_reaction_to_message(message_id.into(), emoji.into())
                    .await?;
                Ok(())
            }

            async fn load_latest_messages(&self) -> ClientResult<MessageResultSet> {
                Ok(self.room.load_latest_messages().await?.into())
            }

            async fn load_messages_before(
                &self,
                before: MessageId,
            ) -> ClientResult<MessageResultSet> {
                Ok(self
                    .room
                    .load_messages_before(&(before.into()))
                    .await?
                    .into())
            }

            async fn load_messages_with_ids(
                &self,
                ids: Vec<MessageId>,
            ) -> ClientResult<Vec<Message>> {
                let ids = ids
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<CoreMessageId>>();
                Ok(self
                    .room
                    .load_messages_with_ids(ids.as_slice())
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect())
            }

            async fn load_unread_messages(&self) -> ClientResult<MessageResultSet> {
                Ok(self.room.load_unread_messages().await?.into())
            }

            async fn set_user_is_composing(&self, is_composing: bool) -> ClientResult<()> {
                self.room.set_user_is_composing(is_composing).await?;
                Ok(())
            }

            async fn load_composing_users(&self) -> ClientResult<Vec<ParticipantBasicInfo>> {
                Ok(self
                    .room
                    .load_composing_users()
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect())
            }

            async fn save_draft(&self, message: Option<String>) -> ClientResult<()> {
                self.room.save_draft(message.as_deref()).await?;
                Ok(())
            }

            async fn load_draft(&self) -> ClientResult<Option<String>> {
                Ok(self.room.load_draft().await?)
            }

            async fn mark_as_read(&self) -> ClientResult<()> {
                self.room.mark_as_read().await?;
                Ok(())
            }

            async fn set_last_read_message(&self, message_id: MessageId) -> ClientResult<()> {
                self.room
                    .set_last_read_message(&(message_id.into()))
                    .await?;
                Ok(())
            }
        }
    };
}

macro_rules! muc_room_impl {
    ($t:ident) => {
        #[uniffi::export(async_runtime = "tokio")]
        #[async_trait::async_trait]
        impl MucRoom for $t {
            fn subject(&self) -> Option<String> {
                self.room.subject()
            }

            async fn set_topic(&self, topic: Option<String>) -> ClientResult<()> {
                self.room.set_topic(topic).await?;
                Ok(())
            }
        }
    };
}

macro_rules! mut_name_impl {
    ($t:ident) => {
        #[uniffi::export(async_runtime = "tokio")]
        #[async_trait::async_trait]
        impl HasMutableName for $t {
            async fn set_name(&self, name: &str) -> ClientResult<()> {
                self.room.set_name(name).await?;
                Ok(())
            }
        }
    };
}

#[uniffi::export]
#[async_trait::async_trait]
pub trait Channel: Send + Sync {
    async fn invite_users(&self, users: Vec<UserId>) -> ClientResult<()>;
}

macro_rules! channel_room_impl {
    ($t:ident) => {
        #[uniffi::export(async_runtime = "tokio")]
        #[async_trait::async_trait]
        impl Channel for $t {
            async fn invite_users(&self, users: Vec<UserId>) -> ClientResult<()> {
                let user_ids = users
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<CoreUserId>>();
                self.room.invite_users(user_ids.iter()).await?;
                Ok(())
            }
        }
    };
}

#[uniffi::export(async_runtime = "tokio")]
impl RoomDirectMessage {
    pub fn is_encryption_enabled(&self) -> bool {
        self.room.encryption_enabled()
    }

    pub async fn set_is_encryption_enabled(&self, enabled: bool) {
        self.room.set_encryption_enabled(enabled).await;
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl RoomGroup {
    pub async fn resend_invites_to_members(&self) -> ClientResult<()> {
        self.room.resend_invites_to_members().await?;
        Ok(())
    }
}

impl From<CoreRoomEnvelope> for RoomEnvelope {
    fn from(value: CoreRoomEnvelope) -> Self {
        match value {
            CoreRoomEnvelope::DirectMessage(room) => {
                RoomEnvelope::DirectMessage(Arc::new(RoomDirectMessage { room }))
            }
            CoreRoomEnvelope::Group(room) => RoomEnvelope::Group(Arc::new(RoomGroup { room })),
            CoreRoomEnvelope::PrivateChannel(room) => {
                RoomEnvelope::PrivateChannel(Arc::new(RoomPrivateChannel { room }))
            }
            CoreRoomEnvelope::PublicChannel(room) => {
                RoomEnvelope::PublicChannel(Arc::new(RoomPublicChannel { room }))
            }
            CoreRoomEnvelope::Generic(room) => {
                RoomEnvelope::Generic(Arc::new(RoomGeneric { room }))
            }
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

#[uniffi::export]
#[async_trait::async_trait]
pub trait MucRoom: Send + Sync {
    fn subject(&self) -> Option<String>;
    async fn set_topic(&self, topic: Option<String>) -> ClientResult<()>;
}

#[uniffi::export]
#[async_trait::async_trait]
pub trait HasMutableName: Send + Sync {
    async fn set_name(&self, name: &str) -> ClientResult<()>;
}

base_room_impl!(RoomDirectMessage);
base_room_impl!(RoomGroup);
base_room_impl!(RoomPrivateChannel);
base_room_impl!(RoomPublicChannel);
base_room_impl!(RoomGeneric);

muc_room_impl!(RoomGroup);
muc_room_impl!(RoomPrivateChannel);
muc_room_impl!(RoomPublicChannel);
muc_room_impl!(RoomGeneric);

mut_name_impl!(RoomPrivateChannel);
mut_name_impl!(RoomPublicChannel);
mut_name_impl!(RoomGeneric);

channel_room_impl!(RoomPrivateChannel);
channel_room_impl!(RoomPublicChannel);
