// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::{BareJid, Jid};
use tracing::{debug, error};
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::message::MessageType;

use prose_proc_macros::InjectDependencies;
use prose_xmpp::mods::chat;
use prose_xmpp::mods::chat::Carbon;
use prose_xmpp::stanza::Message;
use prose_xmpp::Event;

use crate::app::deps::{
    DynClientEventDispatcher, DynConnectedRoomsRepository, DynMessagesRepository,
    DynMessagingService, DynRoomFactory, DynTimeProvider,
};
use crate::app::event_handlers::{XMPPEvent, XMPPEventHandler};
use crate::domain::messaging::models::{MessageLike, TimestampedMessage};
use crate::{ClientEvent, RoomEventType};

#[derive(InjectDependencies)]
pub struct MessagesEventHandler {
    #[inject]
    connected_rooms_repo: DynConnectedRoomsRepository,
    #[inject]
    messages_repo: DynMessagesRepository,
    #[inject]
    messaging_service: DynMessagingService,
    #[inject]
    room_factory: DynRoomFactory,
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
}

impl MessagesEventHandler {
    async fn handle_received_message(&self, message: ReceivedMessage) -> Result<()> {
        let Some(from) = message.sender() else {
            error!("Received message from unknown sender.");
            return Ok(());
        };

        let Some(room) = self.connected_rooms_repo.get(&from) else {
            error!("Received message from sender for which we do not have a room.");
            return Ok(());
        };

        if let ReceivedMessage::Message(message) = &message {
            if let Some(subject) = &message.subject() {
                room.state.write().subject = if subject.is_empty() {
                    None
                } else {
                    Some(subject.to_string())
                };
                return Ok(());
            }
        }

        struct ChatStateEvent {
            state: ChatState,
            from: Jid,
        }

        let mut chat_state: Option<ChatStateEvent> = None;

        if let ReceivedMessage::Message(message) = &message {
            if let (Some(state), Some(from)) = (message.chat_state(), &message.from) {
                chat_state = Some(ChatStateEvent {
                    state,
                    from: if message.type_ == MessageType::Groupchat {
                        from.clone()
                    } else {
                        Jid::Bare(from.to_bare())
                    },
                });
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

        let parsed_message = match parsed_message {
            Ok(message) => Some(message),
            Err(err) => {
                error!("Failed to parse received message: {}", err);
                None
            }
        };

        if parsed_message.is_none() && chat_state.is_none() {
            // Nothing to do…
            return Ok(());
        }

        if let Some(message) = &parsed_message {
            debug!("Caching received message…");
            self.messages_repo.append(&from, &[message]).await?;

            self.client_event_dispatcher
                .dispatch_event(ClientEvent::RoomChanged {
                    room: self.room_factory.build(room.clone()),
                    r#type: RoomEventType::from(message),
                });
        }

        if let Some(chat_state) = chat_state {
            room.state
                .write()
                .set_occupant_chat_state(&chat_state.from, &now, chat_state.state);

            self.client_event_dispatcher
                .dispatch_event(ClientEvent::RoomChanged {
                    room: self.room_factory.build(room.clone()),
                    r#type: RoomEventType::ComposingUsersChanged,
                });
        }

        let Some(message) = parsed_message else {
            return Ok(());
        };

        // Don't send delivery receipts for carbons or anything other than a regular message.
        if message_is_carbon || !message.payload.is_message() {
            return Ok(());
        }

        if let Some(message_id) = message.id.into_original_id() {
            self.messaging_service
                .send_read_receipt(&room.info.jid, &room.info.room_type, &message_id)
                .await?;
        }

        Ok(())
    }

    pub async fn handle_sent_message(&self, message: Message) -> Result<()> {
        let Some(to) = &message.to else {
            error!("Sent message to unknown recipient.");
            return Ok(());
        };

        let to = to.to_bare();

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
            .dispatch_event(ClientEvent::RoomChanged {
                room: self.room_factory.build(room),
                r#type: RoomEventType::from(&message),
            });

        Ok(())
    }
}
