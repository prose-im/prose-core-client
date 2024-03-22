// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;

use anyhow::Result;
use async_trait::async_trait;
use jid::Jid;
use tracing::{debug, error, warn};
use xmpp_parsers::message::MessageType;

use prose_proc_macros::InjectDependencies;
use prose_xmpp::mods::chat::Carbon;
use prose_xmpp::stanza::message::Forwarded;
use prose_xmpp::stanza::Message;

use crate::app::deps::{
    DynAppContext, DynClientEventDispatcher, DynConnectedRoomsReadOnlyRepository,
    DynMessagesRepository, DynSidebarDomainService, DynTimeProvider,
};
use crate::app::event_handlers::{MessageEvent, MessageEventType, ServerEvent, ServerEventHandler};
use crate::domain::messaging::models::{
    MessageLike, MessageLikeError, MessageLikePayload, MessageParser, MessageTargetId,
};
use crate::domain::rooms::models::Room;
use crate::domain::shared::models::{RoomId, UserEndpointId};
use crate::dtos::{OccupantId, ParticipantId};
use crate::infra::xmpp::util::MessageExt;
use crate::ClientRoomEventType;

#[derive(InjectDependencies)]
pub struct MessagesEventHandler {
    #[inject]
    ctx: DynAppContext,
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
    Carbon(Forwarded),
}

enum SentMessage {
    Message(Message),
    Carbon(Forwarded),
}

impl ReceivedMessage {
    pub fn from(&self) -> Option<UserEndpointId> {
        let message = match &self {
            ReceivedMessage::Message(message) => Some(message),
            ReceivedMessage::Carbon(message) => message.stanza.as_ref().map(|m| m.deref()),
        };

        let Some(message) = message else { return None };

        let Some(from) = message.from.clone() else {
            return None;
        };

        if message.is_groupchat_message() {
            let Jid::Full(from) = from else {
                error!("Expected FullJid in received groupchat message");
                return None;
            };
            UserEndpointId::Occupant(from.into())
        } else {
            match from {
                Jid::Bare(from) => UserEndpointId::User(from.into()),
                Jid::Full(from) => UserEndpointId::UserResource(from.into()),
            }
        }
        .into()
    }
}

impl SentMessage {
    pub fn room_id(&self) -> Option<RoomId> {
        let message = match self {
            SentMessage::Message(message) => Some(message),
            SentMessage::Carbon(message) => message.stanza.as_ref().map(|m| m.deref()),
        };

        let Some(message) = message else { return None };

        let Some(to) = message.to.clone() else {
            return None;
        };

        match message.type_ {
            MessageType::Groupchat => RoomId::Muc(to.into_bare().into()),
            _ => RoomId::User(to.into_bare().into()),
        }
        .into()
    }
}

impl MessagesEventHandler {
    async fn handle_message_event(&self, event: MessageEvent) -> Result<()> {
        match event.r#type {
            MessageEventType::Received(mut message) => {
                // When we send a message to a MUC room we'll receive the same message from
                // our JID in the room back to our connected JID.
                // I.e. `from` is 'room@groups.prose.org/me' and `to` is 'me@prose.org/res'
                // In this case we want to treat it as a sent message…
                if message.type_ == MessageType::Groupchat {
                    let Some(Jid::Full(from)) = &message.from else {
                        error!("Expected FullJid in received groupchat message");
                        return Ok(());
                    };

                    let from = OccupantId::from(from.clone());
                    let room_id = from.room_id();

                    if let Some(room) = self.connected_rooms_repo.get(room_id.as_ref()) {
                        // Was the message sent by us?
                        if Some(from) == room.occupant_id() {
                            // Now we'll modify the message so that it looks like other "sent"
                            // messages. Expanding on the example above, we want our
                            // `from` to be 'me@prose.org/res' and our
                            // `to` to be 'room@groups.prose.org/me'.

                            message.from = message.to.take();
                            message.to = Some(room_id.into_bare().into());
                            return self
                                .handle_sent_message(SentMessage::Message(message))
                                .await;
                        }
                    }
                }
                self.handle_received_message(ReceivedMessage::Message(message))
                    .await?
            }
            MessageEventType::Sync(Carbon::Received(message)) => {
                self.handle_received_message(ReceivedMessage::Carbon(message))
                    .await?
            }
            MessageEventType::Sync(Carbon::Sent(message)) => {
                self.handle_sent_message(SentMessage::Carbon(message))
                    .await?
            }
            MessageEventType::Sent(message) => {
                self.handle_sent_message(SentMessage::Message(message))
                    .await?
            }
        }
        Ok(())
    }

    async fn handle_received_message(&self, message: ReceivedMessage) -> Result<()> {
        let Some(from) = message.from() else {
            error!("Received message from unknown sender.");
            return Ok(());
        };

        let room_id = from.to_room_id();

        let parser = MessageParser::new(self.time_provider.now());

        let parsed_message: Result<MessageLike> = match message {
            ReceivedMessage::Message(message) => parser.parse_message(message),
            ReceivedMessage::Carbon(carbon) => parser.parse_forwarded_message(carbon),
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
                .handle_received_message(&message)
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
            error!("Received message from sender ('{room_id}') for which we do not have a room.");
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
            self.dispatch_event_for_message(room, message).await;
        }

        Ok(())
    }

    async fn handle_sent_message(&self, message: SentMessage) -> Result<()> {
        let Some(room_id) = &message.room_id() else {
            error!("Sent message to unknown recipient.");
            return Ok(());
        };

        let Some(room) = self.connected_rooms_repo.get(room_id.as_ref()) else {
            error!("Sent message to recipient ('{room_id}') for which we do not have a room.");
            return Ok(());
        };

        let parser = MessageParser::new(self.time_provider.now());

        let mut parsed_message = match message {
            SentMessage::Message(message) => parser.parse_message(message),
            SentMessage::Carbon(carbon) => parser.parse_forwarded_message(carbon),
        }?;

        // Usually for sent messages the `from` would be our connected JID and the `to` would be
        // the JID of the recipient. For sent groupchat messages the `from` would also be our
        // connected JID and the `to` would be the JID of the room.
        //
        // For received groupchat messages the `from` however would be the JID of the occupant,
        // i.e. 'room@rooms.prose.org/user' and that is what our message parser tries to parse.
        //
        // What we'll receive as the `from` in a parsed message would then be a
        // ParticipantId::Occupant('me@prose.org/res') which is clearly wrong. Which is why we just
        // take our connected jid and plug it into the `from`.
        parsed_message.from = ParticipantId::User(self.ctx.connected_id()?.into_user_id());

        debug!("Caching sent message…");
        let messages = [parsed_message];
        self.messages_repo.append(&room_id, &messages).await?;
        let [parsed_message] = messages;

        self.dispatch_event_for_message(room, parsed_message).await;

        Ok(())
    }

    async fn dispatch_event_for_message(&self, room: Room, message: MessageLike) {
        let event_type = if let Some(target) = message.target {
            let message_id = match target {
                MessageTargetId::MessageId(id) => id,
                MessageTargetId::StanzaId(stanza_id) => {
                    match self
                        .messages_repo
                        .resolve_message_id(&room.room_id, &stanza_id)
                        .await
                    {
                        Ok(Some(id)) => id,
                        Ok(None) => {
                            warn!("Not dispatching event for message with id '{}'. Failed to look up targeted MessageId from StanzaId '{}'.", message.id, stanza_id);
                            return;
                        }
                        Err(err) => {
                            error!("Not dispatching event for message with id '{}'. Encountered error while looking up StanzaId '{}': {}", message.id, stanza_id, err.to_string());
                            return;
                        }
                    }
                }
            };

            if message.payload == MessageLikePayload::Retraction {
                ClientRoomEventType::MessagesDeleted {
                    message_ids: vec![message_id],
                }
            } else {
                ClientRoomEventType::MessagesUpdated {
                    message_ids: vec![message_id],
                }
            }
        } else {
            ClientRoomEventType::MessagesAppended {
                message_ids: vec![message.id.id().as_ref().into()],
            }
        };

        self.client_event_dispatcher
            .dispatch_room_event(room, event_type);
    }
}
