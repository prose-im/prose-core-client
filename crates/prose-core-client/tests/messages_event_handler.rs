// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use chrono::{TimeZone, Utc};
use mockall::{predicate, Sequence};
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::message::MessageType;

use prose_core_client::app::event_handlers::{
    MessageEvent, MessageEventType, MessagesEventHandler, ServerEvent, ServerEventHandler,
};
use prose_core_client::domain::connection::models::ConnectionProperties;
use prose_core_client::domain::messaging::models::{
    MessageLike, MessageLikeBody, MessageLikePayload,
};
use prose_core_client::domain::messaging::services::WrappingMessageIdProvider;
use prose_core_client::domain::rooms::models::{Room, RoomInfo};
use prose_core_client::domain::shared::models::{
    MucId, OccupantId, RoomId, RoomType, UserId, UserResourceId,
};
use prose_core_client::dtos::{
    Availability, MessageId, MessageRemoteId, MessageServerId, ParticipantId,
};
use prose_core_client::test::mock_data::account_jid;
use prose_core_client::test::{ConstantTimeProvider, MockAppDependencies};
use prose_core_client::{muc_id, occupant_id, user_id, user_resource_id, ClientRoomEventType};
use prose_xmpp::mods::chat::Carbon;
use prose_xmpp::stanza::message::stanza_id::StanzaId;
use prose_xmpp::stanza::message::{Forwarded, Reactions};
use prose_xmpp::stanza::muc::MucUser;
use prose_xmpp::stanza::Message;
use prose_xmpp::{bare, full, jid};

#[tokio::test]
async fn test_receiving_message_adds_item_to_sidebar_if_needed() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    deps.message_id_provider = Arc::new(WrappingMessageIdProvider::incrementing("msg-id"));
    let mut seq = Sequence::new();

    let room = Room::group(muc_id!("group@conference.prose.org")).with_name("Group Name");

    deps.messages_repo
        .expect_contains()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::always(),
            predicate::always(),
            predicate::eq(MessageServerId::from("message-id")),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(false) }));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .in_sequence(&mut seq)
            .with(
                predicate::always(),
                predicate::eq(bare!("group@conference.prose.org")),
            )
            .return_once(|_, _| Some(room));
    }

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .in_sequence(&mut seq)
            .with(
                predicate::always(),
                predicate::eq(bare!("group@conference.prose.org")),
            )
            .return_once(|_, _| Some(room));
    }

    deps.sidebar_domain_service
        .expect_handle_received_message()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(RoomId::Muc(muc_id!("group@conference.prose.org"))),
            predicate::function(|msg: &MessageLike| {
                msg.from == ParticipantId::Occupant(occupant_id!("group@conference.prose.org/user"))
            }),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.messages_repo
        .expect_append()
        .once()
        .in_sequence(&mut seq)
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(room),
            predicate::eq(ClientRoomEventType::MessagesAppended {
                message_ids: vec!["msg-id-1".into()],
            }),
        )
        .return_once(|_, _| ());

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Received(
                Message::default()
                    .set_type(MessageType::Groupchat)
                    .set_to(bare!("group@conference.prose.org"))
                    .set_stanza_id(StanzaId {
                        id: "message-id".into(),
                        by: bare!("group@conference.prose.org").into(),
                    })
                    .set_from(jid!("group@conference.prose.org/user"))
                    .set_body("Hello World"),
            ),
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_receiving_message_from_new_contact_creates_room() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    deps.message_id_provider = Arc::new(WrappingMessageIdProvider::incrementing("msg-id"));
    let mut seq = Sequence::new();

    let room = Room::direct_message(user_id!("jane.doe@prose.org"), Availability::Unavailable);

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .in_sequence(&mut seq)
            .with(
                predicate::always(),
                predicate::eq(bare!("jane.doe@prose.org")),
            )
            .return_once(|_, _| Some(room));
    }

    deps.sidebar_domain_service
        .expect_handle_received_message()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(RoomId::User(user_id!("jane.doe@prose.org"))),
            predicate::function(|msg: &MessageLike| {
                msg.from == ParticipantId::User(user_id!("jane.doe@prose.org"))
            }),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.messages_repo
        .expect_contains()
        .once()
        .with(
            predicate::always(),
            predicate::always(),
            predicate::eq(MessageServerId::from("message-id")),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(false) }));

    deps.messages_repo
        .expect_append()
        .once()
        .in_sequence(&mut seq)
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(room),
            predicate::eq(ClientRoomEventType::MessagesAppended {
                message_ids: vec!["msg-id-1".into()],
            }),
        )
        .return_once(|_, _| ());

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Received(
                Message::default()
                    .set_type(MessageType::Chat)
                    .set_to(account_jid())
                    .set_stanza_id(StanzaId {
                        id: "message-id".into(),
                        by: bare!("jane.doe@prose.org").into(),
                    })
                    .set_from(jid!("jane.doe@prose.org"))
                    .set_body("Hello World"),
            ),
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_parses_user_id_from_in_sent_groupchat_message() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    deps.message_id_provider = Arc::new(WrappingMessageIdProvider::incrementing("msg-id"));
    let mut seq = Sequence::new();

    *deps.ctx.connection_properties.write() = Some(ConnectionProperties {
        connection_timestamp: Default::default(),
        connected_jid: user_resource_id!("from@prose.org/res"),
        server_features: Default::default(),
        rooms_caught_up: false,
        decryption_context: None,
    });

    let room = Room::group(muc_id!("room@conference.prose.org"));

    let sent_message = Message::new()
        .set_type(MessageType::Groupchat)
        .set_id("message-id".into())
        .set_stanza_id(StanzaId {
            id: "stanza-id".into(),
            by: bare!("room@conference.prose.org").into(),
        })
        .set_from(full!("from@prose.org/res"))
        .set_to(bare!("room@conference.prose.org"))
        .set_body("Hello World")
        .set_chat_state(Some(ChatState::Active))
        .set_markable();

    let expected_saved_message = MessageLike {
        id: "msg-id-1".into(),
        remote_id: Some("message-id".into()),
        server_id: Some("stanza-id".into()),
        target: None,
        to: Some(bare!("room@conference.prose.org")),
        from: ParticipantId::User(user_id!("from@prose.org")), // Resource should be dropped
        timestamp: Utc.with_ymd_and_hms(2023, 09, 11, 0, 0, 0).unwrap(),
        payload: MessageLikePayload::Message {
            body: MessageLikeBody {
                raw: "Hello World".to_string(),
                html: "<p>Hello World</p>".to_string().into(),
                mentions: vec![],
            },
            attachments: vec![],
            encryption_info: None,
            is_transient: false,
        },
    };

    deps.time_provider = Arc::new(ConstantTimeProvider::ymd(2023, 09, 11));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .in_sequence(&mut seq)
            .with(
                predicate::always(),
                predicate::eq(bare!("room@conference.prose.org")),
            )
            .return_once(|_, _| Some(room));
    }

    deps.messages_repo
        .expect_contains()
        .with(
            predicate::always(),
            predicate::always(),
            predicate::eq(MessageServerId::from("stanza-id")),
        )
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(false) }));

    deps.messages_repo
        .expect_resolve_remote_id_to_message_id()
        .with(
            predicate::always(),
            predicate::always(),
            predicate::eq(MessageRemoteId::from("message-id")),
        )
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(None) }));

    deps.messages_repo
        .expect_append()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::always(),
            predicate::eq(RoomId::Muc(muc_id!("room@conference.prose.org"))),
            predicate::eq([expected_saved_message]),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(room),
            predicate::eq(ClientRoomEventType::MessagesAppended {
                message_ids: vec!["msg-id-1".into()],
            }),
        )
        .return_once(|_, _| ());

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Sent(sent_message),
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_parses_private_message_in_muc_room() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    let mut seq = Sequence::new();
    deps.message_id_provider = Arc::new(WrappingMessageIdProvider::incrementing("msg-id"));

    *deps.ctx.connection_properties.write() = Some(ConnectionProperties {
        connection_timestamp: Default::default(),
        connected_jid: user_resource_id!("user@prose.org/res"),
        server_features: Default::default(),
        rooms_caught_up: false,
        decryption_context: None,
    });

    let room = Room::group(muc_id!("room@conference.prose.org"));

    let received_message = Message::new()
        .set_type(MessageType::Chat)
        .set_id("message-id".into())
        .set_stanza_id(StanzaId {
            id: "stanza-id".into(),
            by: bare!("room@conference.prose.org").into(),
        })
        .set_from(full!("room@conference.prose.org/other-user"))
        .set_to(full!("user@prose.org/res"))
        .set_body("Private Message")
        .add_payload(MucUser::new());

    let expected_saved_message = MessageLike {
        id: "msg-id-1".into(),
        remote_id: Some("message-id".into()),
        server_id: Some("stanza-id".into()),
        target: None,
        to: Some(bare!("user@prose.org")),
        from: ParticipantId::Occupant(occupant_id!("room@conference.prose.org/other-user")),
        timestamp: Utc.with_ymd_and_hms(2023, 09, 11, 0, 0, 0).unwrap(),
        payload: MessageLikePayload::Message {
            body: MessageLikeBody {
                raw: "Private Message".to_string(),
                html: "<p>Private Message</p>".to_string().into(),
                mentions: vec![],
            },
            attachments: vec![],
            encryption_info: None,
            is_transient: true,
        },
    };

    deps.time_provider = Arc::new(ConstantTimeProvider::ymd(2023, 09, 11));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .in_sequence(&mut seq)
            .with(
                predicate::always(),
                predicate::eq(bare!("room@conference.prose.org")),
            )
            .return_once(|_, _| Some(room));
    }

    deps.sidebar_domain_service
        .expect_handle_received_message()
        .once()
        .in_sequence(&mut seq)
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.messages_repo
        .expect_contains()
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(false) }));

    deps.messages_repo
        .expect_append()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::always(),
            predicate::eq(RoomId::Muc(muc_id!("room@conference.prose.org"))),
            predicate::eq([expected_saved_message]),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(room),
            predicate::eq(ClientRoomEventType::MessagesAppended {
                message_ids: vec!["msg-id-1".into()],
            }),
        )
        .return_once(|_, _| ());

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Received(received_message),
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_dispatches_messages_appended_for_new_received_message() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    deps.message_id_provider = Arc::new(WrappingMessageIdProvider::incrementing("msg-id"));

    let room = Room::group(muc_id!("user@prose.org"));

    deps.sidebar_domain_service
        .expect_handle_received_message()
        .once()
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .return_once(|_, _| Some(room));
    }

    deps.messages_repo
        .expect_contains()
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(false) }));

    deps.messages_repo
        .expect_append()
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room),
            predicate::eq(ClientRoomEventType::MessagesAppended {
                message_ids: vec!["msg-id-1".into()],
            }),
        )
        .return_once(|_, _| ());

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Received(
                Message::default()
                    .set_to(account_jid())
                    .set_stanza_id(StanzaId {
                        id: "stanza-id".into(),
                        by: bare!("user@prose.org").into(),
                    })
                    .set_from(jid!("user@prose.org"))
                    .set_body("Hello World"),
            ),
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_dispatches_messages_appended_for_sent_carbon() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    deps.message_id_provider = Arc::new(WrappingMessageIdProvider::incrementing("msg-id"));

    *deps.ctx.connection_properties.write() = Some(ConnectionProperties {
        connection_timestamp: Default::default(),
        connected_jid: user_resource_id!("me@prose.org/res2"),
        server_features: Default::default(),
        rooms_caught_up: false,
        decryption_context: None,
    });

    let room = Room::direct_message(user_id!("user@prose.org"), Availability::Available);

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .return_once(|_, _| Some(room));
    }

    // sidebar_domain_service.handle_received_message should not be called

    deps.messages_repo
        .expect_contains()
        .with(
            predicate::always(),
            predicate::always(),
            predicate::eq(MessageServerId::from("Qiuahv1eo3C222uKhOqjPiW0")),
        )
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(false) }));

    deps.messages_repo
        .expect_resolve_remote_id_to_message_id()
        .with(
            predicate::always(),
            predicate::always(),
            predicate::eq(MessageRemoteId::from("message-id")),
        )
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(None) }));

    deps.messages_repo
        .expect_append()
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room),
            predicate::eq(ClientRoomEventType::MessagesAppended {
                message_ids: vec!["msg-id-1".into()],
            }),
        )
        .return_once(|_, _| ());

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Sync(Carbon::Sent(Forwarded {
                delay: None,
                stanza: Some(Box::new(
                    Message::new()
                        .set_id("message-id".into())
                        .set_type(MessageType::Chat)
                        .set_from(full!("me@prose.org/res1"))
                        .set_to(bare!("user@prose.org"))
                        .set_body("Hello World")
                        .set_chat_state(Some(ChatState::Active))
                        .set_stanza_id(StanzaId {
                            id: "Qiuahv1eo3C222uKhOqjPiW0".into(),
                            by: bare!("user@prose.org").into(),
                        }),
                )),
            })),
        }))
        .await?;

    Ok(())
}

// When we send a message to a MUC room we'll receive the same message back to our
// connected JID. This is what this test is for…
#[tokio::test]
async fn test_dispatches_messages_appended_for_muc_carbon() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    deps.message_id_provider = Arc::new(WrappingMessageIdProvider::incrementing("msg-id"));

    let room = Room::mock(RoomInfo {
        room_id: RoomId::Muc(muc_id!("room@groups.prose.org")),
        user_nickname: "me".to_string(),
        r#type: RoomType::PrivateChannel,
        features: Default::default(),
    });

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .times(2)
            .with(
                predicate::always(),
                predicate::eq(bare!("room@groups.prose.org")),
            )
            .returning(move |_, _| Some(room.clone()));
    }

    // sidebar_domain_service.handle_received_message should not be called

    deps.messages_repo
        .expect_contains()
        .with(
            predicate::always(),
            predicate::always(),
            predicate::eq(MessageServerId::from("Qiuahv1eo3C222uKhOqjPiW0")),
        )
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(false) }));

    deps.messages_repo
        .expect_resolve_remote_id_to_message_id()
        .with(
            predicate::always(),
            predicate::always(),
            predicate::eq(MessageRemoteId::from("message-id")),
        )
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(None) }));

    deps.messages_repo
        .expect_append()
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room),
            predicate::eq(ClientRoomEventType::MessagesAppended {
                message_ids: vec!["msg-id-1".into()],
            }),
        )
        .return_once(|_, _| ());

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Received(
                Message::new()
                    .set_id("message-id".into())
                    .set_type(MessageType::Groupchat)
                    .set_from(full!("room@groups.prose.org/me"))
                    .set_to(full!("me@prose.org/res"))
                    .set_body("Hello World")
                    .set_stanza_id(prose_xmpp::stanza::message::stanza_id::StanzaId {
                        id: "Qiuahv1eo3C222uKhOqjPiW0".into(),
                        by: bare!("user@prose.org").into(),
                    }),
            ),
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_dispatches_messages_updated_for_existing_received_message() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Room::group(muc_id!("user@prose.org"));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .return_once(|_, _| Some(room));
    }

    deps.messages_repo
        .expect_contains()
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(true) }));

    deps.messages_repo
        .expect_append()
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Received(
                Message::default()
                    .set_id("message-id".into())
                    .set_from(jid!("user@prose.org"))
                    .set_to(account_jid())
                    .set_stanza_id(StanzaId {
                        id: "stanza-id".into(),
                        by: bare!("user@prose.org").into(),
                    })
                    .set_body("Hello World"),
            ),
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_looks_up_message_id_when_dispatching_message_event() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Room::group(muc_id!("group@prose.org"));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .times(2)
            .returning(move |_, _| Some(room.clone()));
    }

    deps.messages_repo
        .expect_contains()
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(false) }));

    deps.messages_repo
        .expect_append()
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.messages_repo
        .expect_resolve_server_id_to_message_id()
        .with(
            predicate::always(),
            predicate::eq(RoomId::Muc(muc_id!("group@prose.org"))),
            predicate::eq(MessageServerId::from("stanza-id-100")),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(Some(MessageId::from("message-id-100"))) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room),
            predicate::eq(ClientRoomEventType::MessagesUpdated {
                message_ids: vec!["message-id-100".into()],
            }),
        )
        .return_once(|_, _| ());

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Received(
                Message::default()
                    .set_id("message-id".into())
                    .set_type(MessageType::Groupchat)
                    .set_from(jid!("group@prose.org/user"))
                    .set_to(bare!("group@prose.org"))
                    .set_stanza_id(StanzaId {
                        id: "stanza-id".into(),
                        by: bare!("group@prose.org").into(),
                    })
                    .set_message_reactions(Reactions {
                        id: "stanza-id-100".to_string(),
                        reactions: vec!["🙃".into()],
                    }),
            ),
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_looks_up_message_id_for_sent_groupchat_messages_when_dispatching_message_event(
) -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Room::group(muc_id!("group@prose.org"));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .returning(move |_, _| Some(room.clone()));
    }

    deps.messages_repo
        .expect_contains()
        .with(
            predicate::always(),
            predicate::always(),
            predicate::eq(MessageServerId::from("stanza-id")),
        )
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(false) }));

    deps.messages_repo
        .expect_resolve_remote_id_to_message_id()
        .with(
            predicate::always(),
            predicate::always(),
            predicate::eq(MessageRemoteId::from("message-id")),
        )
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(None) }));

    deps.messages_repo
        .expect_append()
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.messages_repo
        .expect_resolve_server_id_to_message_id()
        .with(
            predicate::always(),
            predicate::eq(RoomId::Muc(muc_id!("group@prose.org"))),
            predicate::eq(MessageServerId::from("stanza-id-100")),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(Some(MessageId::from("message-id-100"))) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room),
            predicate::eq(ClientRoomEventType::MessagesUpdated {
                message_ids: vec!["message-id-100".into()],
            }),
        )
        .return_once(|_, _| ());

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Sent(
                Message::default()
                    .set_id("message-id".into())
                    .set_stanza_id(StanzaId {
                        id: "stanza-id".into(),
                        by: bare!("user@prose.org").into(),
                    })
                    .set_type(MessageType::Groupchat)
                    .set_from(full!("from@prose.org/res"))
                    .set_to(jid!("group@prose.org"))
                    .set_message_reactions(Reactions {
                        id: "stanza-id-100".to_string(),
                        reactions: vec!["🙃".into()],
                    }),
            ),
        }))
        .await?;

    Ok(())
}
