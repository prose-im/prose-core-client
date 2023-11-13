// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

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
    DynAppContext, DynBookmarksService, DynClientEventDispatcher, DynConnectedRoomsRepository,
    DynMessagesRepository, DynMessagingService, DynRoomFactory, DynSidebarRepository,
    DynTimeProvider, DynUserProfileRepository,
};
use crate::app::event_handlers::{XMPPEvent, XMPPEventHandler};
use crate::domain::messaging::models::{MessageLike, MessageLikeError, TimestampedMessage};
use crate::domain::rooms::models::RoomInternals;
use crate::domain::shared::models::{RoomJid, RoomType};
use crate::domain::shared::utils::build_contact_name;
use crate::domain::sidebar::models::{Bookmark, BookmarkType, SidebarItem};
use crate::{ClientEvent, RoomEventType};

#[derive(InjectDependencies)]
pub struct MessagesEventHandler {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    bookmarks_service: DynBookmarksService,
    #[inject]
    connected_rooms_repo: DynConnectedRoomsRepository,
    #[inject]
    messages_repo: DynMessagesRepository,
    #[inject]
    messaging_service: DynMessagingService,
    #[inject]
    room_factory: DynRoomFactory,
    #[inject]
    sidebar_repo: DynSidebarRepository,
    #[inject]
    time_provider: DynTimeProvider,
    #[inject]
    client_event_dispatcher: DynClientEventDispatcher,
    #[inject]
    user_profile_repo: DynUserProfileRepository,
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
            let user_profile = self
                .user_profile_repo
                .get(&from)
                .await
                .ok()
                .map(|maybe_profile| maybe_profile.unwrap_or_default())
                .unwrap_or_default();

            let user_jid = self.ctx.connected_jid()?.into_bare();
            let contact_name = build_contact_name(&from, &user_profile);

            let created_room = Arc::new(RoomInternals::for_direct_message(
                &user_jid,
                &from,
                &contact_name,
            ));
            _ = self.connected_rooms_repo.set(created_room.clone());
            room = Some(created_room);
        }

        let Some(room) = room else {
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
            match self.add_sidebar_item_if_needed(&room).await {
                Ok(_) => (),
                Err(err) => error!("Could not add group to sidebar. {}", err.to_string()),
            }
        }

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::RoomChanged {
                room: self.room_factory.build(room.clone()),
                r#type: RoomEventType::from(&message),
            });

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
            .dispatch_event(ClientEvent::RoomChanged {
                room: self.room_factory.build(room),
                r#type: RoomEventType::from(&message),
            });

        Ok(())
    }
}

impl MessagesEventHandler {
    async fn add_sidebar_item_if_needed(&self, room: &Arc<RoomInternals>) -> Result<()> {
        let bookmark_type = match room.info.room_type {
            RoomType::DirectMessage => BookmarkType::DirectMessage,
            RoomType::Group => BookmarkType::Group,
            _ => return Ok(()),
        };

        if self.sidebar_repo.get(&room.info.jid).is_some() {
            return Ok(());
        }

        let bookmark_name = room
            .state
            .read()
            .name
            .clone()
            .unwrap_or("Untitled Conversation".to_string());

        self.bookmarks_service
            .save_bookmark(&Bookmark {
                name: bookmark_name.clone(),
                jid: room.info.jid.clone(),
                r#type: bookmark_type.clone(),
                is_favorite: false,
                in_sidebar: true,
            })
            .await?;

        self.sidebar_repo.put(&SidebarItem {
            name: bookmark_name,
            jid: room.info.jid.clone(),
            r#type: bookmark_type,
            is_favorite: false,
            error: None,
        });

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }
}
