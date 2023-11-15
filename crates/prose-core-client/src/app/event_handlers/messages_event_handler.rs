// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use tracing::{debug, error};
use xmpp_parsers::message::MessageType;

use prose_proc_macros::InjectDependencies;
use prose_xmpp::mods::chat;
use prose_xmpp::mods::chat::Carbon;
use prose_xmpp::stanza::Message;
use prose_xmpp::Event;

use crate::app::deps::{
    DynAppContext, DynClientEventDispatcher, DynConnectedRoomsReadOnlyRepository,
    DynMessagesRepository, DynMessagingService, DynSidebarDomainService, DynTimeProvider,
};
use crate::app::event_handlers::{XMPPEvent, XMPPEventHandler};
use crate::domain::messaging::models::{MessageLike, MessageLikeError, TimestampedMessage};
use crate::domain::rooms::services::{CreateOrEnterRoomRequest, CreateRoomType};
use crate::domain::shared::models::RoomJid;
use crate::RoomEventType;

#[derive(InjectDependencies)]
pub struct MessagesEventHandler {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    connected_rooms_repo: DynConnectedRoomsReadOnlyRepository,
    #[inject]
    messages_repo: DynMessagesRepository,
    #[inject]
    messaging_service: DynMessagingService,
    #[inject]
    sidebar_domain_service: DynSidebarDomainService,
    #[inject]
    time_provider: DynTimeProvider,
    #[inject]
    client_event_dispatcher: DynClientEventDispatcher,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl XMPPEventHandler for MessagesEventHandler {
    fn name(&self) -> &'static str {
        "messages"
    }

    async fn handle_event(&self, event: XMPPEvent) -> Result<Option<XMPPEvent>> {
        match event {
            Event::Chat(event) => match event {
                chat::Event::Message(message) => {
                    self.handle_received_message(ReceivedMessage::Message(message))
                        .await?;
                    Ok(None)
                }
                chat::Event::Carbon(carbon) => {
                    self.handle_received_message(ReceivedMessage::Carbon(carbon))
                        .await?;
                    Ok(None)
                }
                chat::Event::Sent(message) => {
                    self.handle_sent_message(message).await?;
                    Ok(None)
                }
                _ => Ok(Some(Event::Chat(event))),
            },
            _ => Ok(Some(event)),
        }
    }
}

enum ReceivedMessage {
    Message(Message),
    Carbon(Carbon),
}

impl ReceivedMessage {
    pub fn is_carbon(&self) -> bool {
        match self {
            Self::Message(_) => false,
            Self::Carbon(_) => true,
        }
    }

    pub fn sender(&self) -> Option<BareJid> {
        match &self {
            ReceivedMessage::Message(message) => message.from.as_ref().map(|jid| jid.to_bare()),
            ReceivedMessage::Carbon(Carbon::Received(message)) => message
                .stanza
                .as_ref()
                .and_then(|message| message.from.as_ref())
                .map(|jid| jid.to_bare()),
            ReceivedMessage::Carbon(Carbon::Sent(message)) => message
                .stanza
                .as_ref()
                .and_then(|message| message.to.as_ref())
                .map(|jid| jid.to_bare()),
        }
    }

    pub fn r#type(&self) -> Option<MessageType> {
        match self {
            ReceivedMessage::Message(message) => Some(message.type_.clone()),
            ReceivedMessage::Carbon(Carbon::Received(message)) => {
                message.stanza.as_ref().map(|m| m.type_.clone())
            }
            ReceivedMessage::Carbon(Carbon::Sent(message)) => {
                message.stanza.as_ref().map(|m| m.type_.clone())
            }
        }
    }
}

impl MessagesEventHandler {
    async fn handle_received_message(&self, message: ReceivedMessage) -> Result<()> {
        let Some(from) = message.sender() else {
            error!("Received message from unknown sender.");
            return Ok(());
        };

        let from = RoomJid::from(from);

        let mut room = self.connected_rooms_repo.get(&from);

        if room.is_none() && message.r#type() == Some(MessageType::Chat) {
            self.sidebar_domain_service
                .insert_item_by_creating_or_joining_room(CreateOrEnterRoomRequest::Create {
                    service: self.ctx.muc_service()?,
                    room_type: CreateRoomType::DirectMessage {
                        participant: from.clone().into_inner(),
                    },
                })
                .await?;
            room = self.connected_rooms_repo.get(&from);
        }

        let Some(room) = room else {
            error!("Received message from sender for which we do not have a room.");
            return Ok(());
        };

        if let ReceivedMessage::Message(message) = &message {
            if let Some(subject) = &message.subject() {
                room.set_topic((!subject.is_empty()).then_some(subject));
                return Ok(());
            }
        }

        let message_is_carbon = message.is_carbon();
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

        debug!("Caching received message…");
        self.messages_repo.append(&from, &[&message]).await?;

        if message.payload.is_message() {
            match self
                .sidebar_domain_service
                .insert_item_for_received_message_if_needed(&from)
                .await
            {
                Ok(_) => (),
                Err(err) => error!("Could not add group to sidebar. {}", err.to_string()),
            }
        }

        self.client_event_dispatcher
            .dispatch_room_event(room.clone(), RoomEventType::from(&message));

        // Don't send delivery receipts for carbons or anything other than a regular message.
        if message_is_carbon || !message.payload.is_message() {
            return Ok(());
        }

        if let Some(message_id) = message.id.into_original_id() {
            self.messaging_service
                .send_read_receipt(&room.jid, &room.r#type, &message_id)
                .await?;
        }

        Ok(())
    }

    pub async fn handle_sent_message(&self, message: Message) -> Result<()> {
        let Some(to) = &message.to else {
            error!("Sent message to unknown recipient.");
            return Ok(());
        };

        let to = RoomJid::from(to.to_bare());

        let Some(room) = self.connected_rooms_repo.get(&to) else {
            error!("Sent message to recipient for which we do not have a room.");
            return Ok(());
        };

        let message = MessageLike::try_from(TimestampedMessage {
            message,
            timestamp: self.time_provider.now(),
        })?;

        debug!("Caching sent message…");
        self.messages_repo.append(&to, &[&message]).await?;

        self.client_event_dispatcher
            .dispatch_room_event(room, RoomEventType::from(&message));

        Ok(())
    }
}
