// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use chrono::TimeDelta;
use tracing::{error, info, warn};
use xmpp_parsers::message::MessageType;

use prose_proc_macros::InjectDependencies;
use prose_xmpp::mods::chat::Carbon;
use prose_xmpp::stanza::message::Forwarded;
use prose_xmpp::stanza::Message;

use crate::app::deps::{
    DynAppContext, DynClientEventDispatcher, DynConnectedRoomsReadOnlyRepository,
    DynEncryptionDomainService, DynMessageIdProvider, DynMessagesRepository,
    DynOfflineMessagesRepository, DynSidebarDomainService, DynTimeProvider,
};
use crate::app::event_handlers::{MessageEvent, MessageEventType, ServerEvent, ServerEventHandler};
use crate::domain::messaging::models::{
    MessageId, MessageLike, MessageLikeError, MessageLikePayload, MessageParser, MessageTargetId,
};
use crate::domain::rooms::models::Room;
use crate::domain::shared::models::{AccountId, ConnectionState, RoomId, UserEndpointId};
use crate::dtos::{MessageRemoteId, OccupantId, ParticipantId};
use crate::infra::xmpp::util::MessageExt;
use crate::ClientRoomEventType;

#[derive(InjectDependencies)]
pub struct MessagesEventHandler {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    connected_rooms_repo: DynConnectedRoomsReadOnlyRepository,
    #[inject]
    encryption_domain_service: DynEncryptionDomainService,
    #[inject]
    message_id_provider: DynMessageIdProvider,
    #[inject]
    messages_repo: DynMessagesRepository,
    #[inject]
    sidebar_domain_service: DynSidebarDomainService,
    #[inject]
    time_provider: DynTimeProvider,
    #[inject]
    client_event_dispatcher: DynClientEventDispatcher,
    #[inject]
    offline_messages_repo: DynOfflineMessagesRepository,
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
                // We're collecting offline messages that we're receiving while we're
                // still connecting. This is important since received messages can trigger the
                // creation of rooms, but some of these rooms rely on data that is still being
                // determined during the connection process, like server features. The collected
                // messages will be applied by the ConnectionService after the connection was
                // complete and successful.
                if self.ctx.connection_state() != ConnectionState::Connected {
                    info!("Caching offline message…");
                    self.offline_messages_repo.push(event);
                    return Ok(None);
                }
                self.handle_message_event(event).await?;
            }
            _ => return Ok(Some(event)),
        }
        Ok(None)
    }
}

impl MessageEventType {
    fn message(&self) -> &Message {
        match self {
            MessageEventType::Received(message) => message,
            MessageEventType::Sent(message) => message,
            MessageEventType::Sync(Carbon::Received(carbon)) => carbon.message.as_ref(),
            MessageEventType::Sync(Carbon::Sent(carbon)) => carbon.message.as_ref(),
        }
    }
}

enum MessageOrCarbon {
    Message(Message),
    Carbon(Forwarded),
}

impl MessageOrCarbon {
    fn message(&self) -> &Message {
        match self {
            Self::Message(message) => message,
            Self::Carbon(carbon) => carbon.message.as_ref(),
        }
    }

    pub fn from(&self) -> Option<UserEndpointId> {
        self.message().sender()
    }

    pub fn room_id(&self) -> Option<RoomId> {
        self.message().room_id()
    }

    pub fn remote_id(&self) -> Option<MessageRemoteId> {
        self.message().id.clone().map(MessageRemoteId::from)
    }
}

impl MessagesEventHandler {
    async fn handle_message_event(&self, event: MessageEvent) -> Result<()> {
        let account = self.ctx.connected_account()?;

        // Skip known messages…
        {
            let msg = event.r#type.message();
            if let (Some(room_id), Some(server_id)) = (msg.room_id(), msg.server_id()) {
                if self
                    .messages_repo
                    .contains(&account, &room_id, &server_id)
                    .await?
                {
                    // We've seen this message already…
                    return Ok(());
                }
            }
        }

        match event.r#type {
            MessageEventType::Received(mut message) => {
                // When we send a message to a MUC room we'll receive the same message from
                // our JID in the room back to our connected JID.
                // I.e. `from` is 'room@groups.prose.org/me' and `to` is 'me@prose.org/res'
                // In this case we want to treat it as a sent message…
                if message.type_ == MessageType::Groupchat {
                    let Some(from) = message
                        .from
                        .as_ref()
                        .and_then(|from| from.try_as_full().ok())
                        .cloned()
                    else {
                        error!("Expected FullJid in received groupchat message");
                        return Ok(());
                    };

                    let from = OccupantId::from(from.clone());
                    let room_id = from.room_id();

                    if let Some(room) = self.connected_rooms_repo.get(&account, room_id.as_ref()) {
                        // Was the message sent by us?
                        if Some(from) == room.occupant_id() {
                            // Now we'll modify the message so that it looks like other "sent"
                            // messages. Expanding on the example above, we want our
                            // `from` to be 'me@prose.org/res' and our
                            // `to` to be 'room@groups.prose.org/me'.

                            message.from = message.to.take();
                            message.to = Some(room_id.into_bare().into());
                            return self
                                .handle_sent_message(account, MessageOrCarbon::Message(message))
                                .await;
                        }
                    }
                }
                self.handle_received_message(account, MessageOrCarbon::Message(message))
                    .await?
            }
            MessageEventType::Sync(Carbon::Received(message)) => {
                self.handle_received_message(account, MessageOrCarbon::Carbon(message))
                    .await?
            }
            MessageEventType::Sync(Carbon::Sent(message)) => {
                self.handle_sent_message(account, MessageOrCarbon::Carbon(message))
                    .await?
            }
            MessageEventType::Sent(message) => {
                self.handle_sent_message(account, MessageOrCarbon::Message(message))
                    .await?
            }
        }
        Ok(())
    }

    async fn handle_received_message(
        &self,
        account: AccountId,
        message: MessageOrCarbon,
    ) -> Result<()> {
        let Some(from) = message.from() else {
            error!("Received message from unknown sender.");
            return Ok(());
        };

        let room_id = from.to_room_id();
        let room = self.connected_rooms_repo.get(&account, room_id.as_ref());
        let now = self.time_provider.now();

        let server_time = now
            + room
                .as_ref()
                .map(|room| room.features.server_time_offset)
                .or_else(|| {
                    self.ctx
                        .server_features()
                        .map(|f| f.server_time_offset)
                        .ok()
                })
                .unwrap_or_else(|| TimeDelta::zero());

        let parser = MessageParser::new(
            self.message_id_provider.new_id(),
            room.clone(),
            server_time,
            self.encryption_domain_service.clone(),
            self.ctx.decryption_context(),
        );

        let parsed_message: Result<MessageLike> = match message {
            MessageOrCarbon::Message(message) => parser.parse_message(message).await,
            MessageOrCarbon::Carbon(carbon) => parser.parse_forwarded_message(carbon).await,
        };

        let message = match parsed_message {
            Ok(message) => message,
            Err(err) => {
                return match err.downcast_ref::<MessageLikeError>() {
                    Some(MessageLikeError::NoPayload) => Ok(()),
                    None => {
                        error!("Failed to parse received message: {:?}", err);
                        Ok(())
                    }
                }
            }
        };

        if message.payload.is_message() {
            _ = self
                .sidebar_domain_service
                .handle_received_message(&room_id, &message)
                .await
                .inspect_err(|err| {
                    error!(
                        "Could not insert sidebar item for message. {}",
                        err.to_string()
                    )
                });
        }

        let Some(room) = room else {
            error!("Received message from sender ('{room_id}') for which we do not have a room.");

            // Save the message regardless. The SidebarDomainService should have created a room by
            // now, but we still don't want to send a messagesUpdated event for a fresh room.
            self.messages_repo
                .append(&account, &room_id, &[message])
                .await?;

            return Ok(());
        };

        self.save_message_and_dispatch_event(&account, room, message)
            .await?;
        Ok(())
    }

    async fn handle_sent_message(
        &self,
        account: AccountId,
        message: MessageOrCarbon,
    ) -> Result<()> {
        let Some(room_id) = &message.room_id() else {
            error!("Sent message to unknown recipient.");
            return Ok(());
        };

        let Some(room) = self.connected_rooms_repo.get(&account, room_id.as_ref()) else {
            error!("Sent message to recipient ('{room_id}') for which we do not have a room.");
            return Ok(());
        };

        let existing_message_id = match message.remote_id() {
            Some(remote_id) => self
                .messages_repo
                .resolve_remote_id(&account, &room_id, &remote_id)
                .await?
                .map(|t| t.id),
            None => None,
        };

        let is_update = existing_message_id.is_some();

        let parser = MessageParser::new(
            existing_message_id.unwrap_or_else(|| self.message_id_provider.new_id()),
            Some(room.clone()),
            room.features
                .local_time_to_server_time(self.time_provider.now()),
            self.encryption_domain_service.clone(),
            self.ctx.decryption_context(),
        );

        let mut parsed_message = match message {
            MessageOrCarbon::Message(message) => parser.parse_message(message).await,
            MessageOrCarbon::Carbon(carbon) => parser.parse_forwarded_message(carbon).await,
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
        parsed_message.from = ParticipantId::User(account.to_user_id());

        // We had this message saved before without a StanzaId, so we'll save it again with
        // the StanzaId but won't dispatch an event since this part is irrelevant for the UI.
        if is_update {
            self.messages_repo
                .append(&account, &room_id, &[parsed_message])
                .await?;
            return Ok(());
        }

        self.save_message_and_dispatch_event(&account, room, parsed_message)
            .await?;
        Ok(())
    }

    async fn save_message_and_dispatch_event(
        &self,
        account: &AccountId,
        room: Room,
        message: MessageLike,
    ) -> Result<()> {
        let messages = [message];
        self.messages_repo
            .append(&account, &room.room_id, &messages)
            .await?;
        let [message] = messages;

        let event_type = if let Some(target_id) = message.payload.target_id() {
            let Some(message_id) = self
                .resolve_message_target_id(account, &room.room_id, target_id.clone())
                .await
            else {
                return Ok(());
            };

            if matches!(message.payload, MessageLikePayload::Retraction { .. }) {
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
                message_ids: vec![message.id.clone()],
            }
        };

        self.client_event_dispatcher
            .dispatch_room_event(room, event_type);

        Ok(())
    }

    async fn resolve_message_target_id(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        target_id: MessageTargetId,
    ) -> Option<MessageId> {
        let result = match &target_id {
            MessageTargetId::RemoteId(remote_id) => {
                self.messages_repo
                    .resolve_remote_id(account, &room_id, remote_id)
                    .await
            }
            MessageTargetId::ServerId(server_id) => {
                self.messages_repo
                    .resolve_server_id(account, &room_id, server_id)
                    .await
            }
        };

        match result {
            Ok(Some(triple)) => Some(triple.id),
            Ok(None) => {
                warn!("Not dispatching event for message. Failed to look up targeted MessageId from {:?}.", target_id);
                None
            }
            Err(err) => {
                error!("Not dispatching event for message. Encountered error while looking up {:?}: {}", target_id, err.to_string());
                None
            }
        }
    }
}
