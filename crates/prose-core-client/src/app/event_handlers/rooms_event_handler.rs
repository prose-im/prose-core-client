// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::{BareJid, Jid};
use tracing::{debug, error, info};
use xmpp_parsers::message::MessageType;
use xmpp_parsers::muc::MucUser;
use xmpp_parsers::presence::Presence;

use prose_proc_macros::InjectDependencies;
use prose_xmpp::mods::chat::Carbon;
use prose_xmpp::mods::{bookmark, bookmark2, chat, muc, status};
use prose_xmpp::stanza::message::ChatState;
use prose_xmpp::stanza::Message;
use prose_xmpp::{ns, Event};

use crate::app::deps::{
    DynAppServiceDependencies, DynConnectedRoomsRepository, DynMessagesRepository,
    DynMessagingService, DynRoomFactory, DynRoomsDomainService,
};
use crate::app::event_handlers::{XMPPEvent, XMPPEventHandler};
use crate::client_event::RoomEventType;
use crate::domain::messaging::models::{MessageLike, MessageLikePayload, TimestampedMessage};
use crate::domain::rooms::services::{CreateOrEnterRoomRequest, CreateOrEnterRoomRequestType};
use crate::ClientEvent;

#[derive(InjectDependencies)]
pub(crate) struct RoomsEventHandler {
    #[inject]
    app_service: DynAppServiceDependencies,
    #[inject]
    connected_rooms_repo: DynConnectedRoomsRepository,
    #[inject]
    messages_repo: DynMessagesRepository,
    #[inject]
    messaging_service: DynMessagingService,
    #[inject]
    room_factory: DynRoomFactory,
    #[inject]
    rooms_domain_service: DynRoomsDomainService,
}

#[async_trait]
impl XMPPEventHandler for RoomsEventHandler {
    fn name(&self) -> &'static str {
        "rooms"
    }

    async fn handle_event(&self, event: XMPPEvent) -> Result<Option<XMPPEvent>> {
        match event {
            Event::Status(event) => match event {
                status::Event::Presence(presence) => {
                    self.presence_did_change(presence).await?;
                    Ok(None)
                }
                _ => Ok(Some(Event::Status(event))),
            },
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
            Event::MUC(event) => match event {
                muc::Event::DirectInvite { from, invite } => {
                    self.handle_invite(invite.jid, invite.password).await?;
                    Ok(None)
                }
                muc::Event::MediatedInvite { from, invite } => {
                    self.handle_invite(from.to_bare(), invite.password).await?;
                    Ok(None)
                }
            },
            Event::Bookmark(event) => match event {
                bookmark::Event::BookmarksChanged {
                    bookmarks: _bookmarks,
                } => {
                    // TODO: Handle changed bookmarks
                    Ok(None)
                }
            },

            Event::Bookmark2(event) => match event {
                bookmark2::Event::BookmarksPublished {
                    bookmarks: _bookmarks,
                } => {
                    // TODO: Handle changed bookmarks
                    Ok(None)
                }
                bookmark2::Event::BookmarksRetracted { jids: _jids } => {
                    // TODO: Handle changed bookmarks
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

impl RoomsEventHandler {
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
            if let Some(subject) = &message.subject {
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
            if let (Some(state), Some(from)) = (&message.chat_state, &message.from) {
                chat_state = Some(ChatStateEvent {
                    state: state.clone(),
                    from: if message.r#type == MessageType::Groupchat {
                        from.clone()
                    } else {
                        Jid::Bare(from.to_bare())
                    },
                });
            }
        }

        let message_is_carbon = message.is_carbon();
        let now = self.app_service.time_provider.now();

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

            self.app_service
                .event_dispatcher
                .dispatch_event(ClientEvent::RoomChanged {
                    room: self.room_factory.build(room.clone()),
                    r#type: RoomEventType::from(message),
                });
        }

        if let Some(chat_state) = chat_state {
            room.state
                .write()
                .set_occupant_chat_state(&chat_state.from, &now, chat_state.state);

            self.app_service
                .event_dispatcher
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
            timestamp: self.app_service.time_provider.now(),
        })?;

        debug!("Caching sent message…");
        self.messages_repo.append(&to, &[&message]).await?;

        self.app_service
            .event_dispatcher
            .dispatch_event(ClientEvent::RoomChanged {
                room: self.room_factory.build(room),
                r#type: RoomEventType::from(&message),
            });

        Ok(())
    }

    async fn presence_did_change(&self, presence: Presence) -> Result<()> {
        let Some(to) = presence.to else {
            error!("Received presence from unknown user.");
            return Ok(());
        };

        let to = to.into_bare();

        let Some(room) = self.connected_rooms_repo.get(&to) else {
            error!("Received presence from user for which we do not have a room.");
            return Ok(());
        };

        let Some(from) = &presence.from else {
            return Ok(());
        };

        let Some(muc_user) = presence
            .payloads
            .into_iter()
            .filter_map(|payload| {
                if !payload.is("x", ns::MUC_USER) {
                    return None;
                }
                MucUser::try_from(payload).ok()
            })
            .take(1)
            .next()
        else {
            return Ok(());
        };

        // Let's try to pull out the real jid of our user…
        let Some((jid, affiliation)) = muc_user
            .items
            .into_iter()
            .filter_map(|item| item.jid.map(|jid| (jid, item.affiliation)))
            .take(1)
            .next()
        else {
            return Ok(());
        };

        info!("Received real jid for {}: {}", from, jid);
        room.state
            .write()
            .insert_occupant(from, Some(&jid.into_bare()), &affiliation);

        Ok(())
    }

    async fn handle_invite(&self, room_jid: BareJid, password: Option<String>) -> Result<()> {
        info!("Joining room {} after receiving invite…", room_jid);

        self.rooms_domain_service
            .create_or_join_room(CreateOrEnterRoomRequest {
                r#type: CreateOrEnterRoomRequestType::Join {
                    room_jid,
                    nickname: None,
                    password,
                },
                save_bookmark: true,
                notify_delegate: true,
            })
            .await?;

        Ok(())
    }
}

impl From<&MessageLike> for RoomEventType {
    fn from(message: &MessageLike) -> Self {
        if let Some(ref target) = message.target {
            if message.payload == MessageLikePayload::Retraction {
                Self::MessagesDeleted {
                    message_ids: vec![target.as_ref().into()],
                }
            } else {
                Self::MessagesUpdated {
                    message_ids: vec![target.as_ref().into()],
                }
            }
        } else {
            Self::MessagesAppended {
                message_ids: vec![message.id.id().as_ref().into()],
            }
        }
    }
}
