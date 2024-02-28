// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use anyhow::Result;
use async_trait::async_trait;
use jid::Jid;
use tracing::{debug, error};
use xmpp_parsers::message::MessageType;

use prose_proc_macros::InjectDependencies;
use prose_xmpp::mods::chat::Carbon;
use prose_xmpp::stanza::Message;

use crate::app::deps::{
    DynClientEventDispatcher, DynConnectedRoomsReadOnlyRepository, DynMessagesRepository,
    DynSidebarDomainService, DynTimeProvider,
};
use crate::app::event_handlers::{MessageEvent, MessageEventType, ServerEvent, ServerEventHandler};
use crate::domain::messaging::models::{MessageLike, MessageLikeError, TimestampedMessage};
use crate::domain::shared::models::{MucId, RoomId, UserEndpointId};
use crate::dtos::UserId;
use crate::ClientRoomEventType;

#[derive(InjectDependencies)]
pub struct MessagesEventHandler {
    #[inject]
    connected_rooms_repo: DynConnectedRoomsReadOnlyRepository,
    #[inject]
    messages_repo: DynMessagesRepository,
    #[inject]
    sidebar_domain_service: DynSidebarDomainService,
    #[inject]
    time_provider: DynTimeProvider,
    #[inject]
    client_event_dispatcher: DynClientEventDispatcher,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ServerEventHandler for MessagesEventHandler {
    fn name(&self) -> &'static str {
        "messages"
    }

    async fn handle_event(&self, event: ServerEvent) -> Result<Option<ServerEvent>> {
        match event {
            ServerEvent::Message(event) => {
                self.handle_message_event(event).await?;
            }
            _ => return Ok(Some(event)),
        }
        Ok(None)
    }
}

enum ReceivedMessage {
    Message(Message),
    Carbon(Carbon),
}

impl ReceivedMessage {
    pub fn sender(&self) -> Option<UserEndpointId> {
        let message = match &self {
            ReceivedMessage::Message(message) => Some(message),
            ReceivedMessage::Carbon(Carbon::Received(message)) => {
                message.stanza.as_ref().map(|b| b.deref())
            }
            ReceivedMessage::Carbon(Carbon::Sent(message)) => {
                message.stanza.as_ref().map(|b| b.deref())
            }
        };

        let Some(message) = message else { return None };

        let Some(from) = message.from.clone() else {
            return None;
        };

        match message.type_ {
            MessageType::Groupchat => {
                let Jid::Full(from) = from else {
                    error!("Expected FullJid in ChatState");
                    return None;
                };
                UserEndpointId::Occupant(from.into())
            }
            _ => match from {
                Jid::Bare(from) => UserEndpointId::User(from.into()),
                Jid::Full(from) => UserEndpointId::UserResource(from.into()),
            },
        }
        .into()
    }
}

impl MessagesEventHandler {
    async fn handle_message_event(&self, event: MessageEvent) -> Result<()> {
        match event.r#type {
            MessageEventType::Received(message) => {
                self.handle_received_message(ReceivedMessage::Message(message))
                    .await?
            }
            MessageEventType::Sync(carbon) => {
                self.handle_received_message(ReceivedMessage::Carbon(carbon))
                    .await?
            }
            MessageEventType::Sent(message) => self.handle_sent_message(message).await?,
        }
        Ok(())
    }

    async fn handle_received_message(&self, message: ReceivedMessage) -> Result<()> {
        let Some(from) = message.sender() else {
            error!("Received message from unknown sender.");
            return Ok(());
        };

        let room_id = from.to_room_id();
        let now = self.time_provider.now();

        let parsed_message: Result<MessageLike> = match message {
            ReceivedMessage::Message(message) => MessageLike::try_from(TimestampedMessage {
                message,
                timestamp: now.into(),
            }),
            ReceivedMessage::Carbon(carbon) => MessageLike::try_from(TimestampedMessage {
                message: carbon,
                timestamp: now.into(),
            }),
        };

        let message = match parsed_message {
            Ok(message) => message,
            Err(err) => {
                return match err.downcast_ref::<MessageLikeError>() {
                    Some(MessageLikeError::NoPayload) => Ok(()),
                    None => {
                        error!("Failed to parse received message: {}", err);
                        Ok(())
                    }
                }
            }
        };

        if message.payload.is_message() {
            match self
                .sidebar_domain_service
                .handle_received_message(&from)
                .await
            {
                Ok(_) => (),
                Err(err) => error!(
                    "Could not insert sidebar item for message. {}",
                    err.to_string()
                ),
            }
        }

        let Some(room) = self.connected_rooms_repo.get(room_id.as_ref()) else {
            error!("Received message from sender for which we do not have a room.");
            return Ok(());
        };

        let is_message_update = if let Some(message_id) = message.id.original_id() {
            self.messages_repo
                .contains(message_id)
                .await
                .unwrap_or(false)
        } else {
            false
        };

        debug!("Caching received message…");
        let messages = [message];
        self.messages_repo.append(&room_id, &messages).await?;

        let [message] = messages;

        if is_message_update {
            self.client_event_dispatcher.dispatch_room_event(
                room.clone(),
                ClientRoomEventType::MessagesUpdated {
                    message_ids: vec![message.id.id().clone()],
                },
            )
        } else {
            self.client_event_dispatcher
                .dispatch_room_event(room.clone(), ClientRoomEventType::from(&message));
        }

        Ok(())
    }

    async fn handle_sent_message(&self, mut message: Message) -> Result<()> {
        let Some(to) = &message.to else {
            error!("Sent message to unknown recipient.");
            return Ok(());
        };

        let to = match message.type_ {
            MessageType::Groupchat => RoomId::Muc(MucId::from(to.to_bare())),
            _ => RoomId::User(UserId::from(to.to_bare())),
        };

        let Some(room) = self.connected_rooms_repo.get(to.as_ref()) else {
            error!("Sent message to recipient for which we do not have a room.");
            return Ok(());
        };

        // For the purpose of parsing our sent message into a `MessageLike` let's treat it as
        // a regular 'chat' message. This way the 'from' attribute will be parsed into
        // a ParticipantId::User instead of a ParticipantId::Occupant which is what we want.
        // Otherwise the ParticipantId::Occupant would contain our (real) FullJid, not our JID in
        // the room.
        message.type_ = MessageType::Chat;
        let message = MessageLike::try_from(TimestampedMessage {
            message,
            timestamp: self.time_provider.now(),
        })?;

        debug!("Caching sent message…");
        let messages = [message];
        self.messages_repo.append(&to, &messages).await?;
        let [message] = messages;

        self.client_event_dispatcher
            .dispatch_room_event(room, ClientRoomEventType::from(&message));

        Ok(())
    }
}
