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
use prose_core_client::domain::messaging::models::{MessageLike, MessageLikePayload};
use prose_core_client::domain::rooms::models::Room;
use prose_core_client::domain::shared::models::{OccupantId, RoomId, UserEndpointId, UserId};
use prose_core_client::dtos::{Availability, MessageId, ParticipantId};
use prose_core_client::test::{ConstantTimeProvider, MockAppDependencies};
use prose_core_client::{occupant_id, room_id, user_id, ClientRoomEventType};
use prose_xmpp::stanza::Message;
use prose_xmpp::{bare, full, jid};

#[tokio::test]
async fn test_receiving_message_adds_item_to_sidebar_if_needed() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    let mut seq = Sequence::new();

    let room = Room::group(room_id!("group@conference.prose.org")).with_name("Group Name");

    deps.sidebar_domain_service
        .expect_insert_item_for_received_message_if_needed()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(UserEndpointId::Occupant(occupant_id!(
            "group@conference.prose.org/jane.doe"
        ))))
        .return_once(|_| Box::pin(async { Ok(()) }));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .in_sequence(&mut seq)
            .with(predicate::eq(room_id!("group@conference.prose.org")))
            .return_once(|_| Some(room));
    }

    deps.messages_repo
        .expect_contains()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(MessageId::from("message-id")))
        .return_once(|_| Box::pin(async { Ok(false) }));

    deps.messages_repo
        .expect_append()
        .once()
        .in_sequence(&mut seq)
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(room),
            predicate::eq(ClientRoomEventType::MessagesAppended {
                message_ids: vec!["message-id".into()],
            }),
        )
        .return_once(|_, _| ());

    deps.messaging_service
        .expect_send_read_receipt()
        .once()
        .in_sequence(&mut seq)
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Received(
                Message::default()
                    .set_type(MessageType::Groupchat)
                    .set_id("message-id".into())
                    .set_from(jid!("group@conference.prose.org/jane.doe"))
                    .set_body("Hello World"),
            ),
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_receiving_message_from_new_contact_creates_room() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    let mut seq = Sequence::new();

    let room = Room::direct_message(user_id!("jane.doe@prose.org"), Availability::Unavailable);

    deps.sidebar_domain_service
        .expect_insert_item_for_received_message_if_needed()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(UserEndpointId::User(user_id!(
            "jane.doe@prose.org"
        ))))
        .return_once(|_| Box::pin(async { Ok(()) }));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .in_sequence(&mut seq)
            .with(predicate::eq(room_id!("jane.doe@prose.org")))
            .return_once(|_| Some(room));
    }

    deps.messages_repo
        .expect_contains()
        .once()
        .with(predicate::eq(MessageId::from("message-id")))
        .return_once(|_| Box::pin(async { Ok(false) }));

    deps.messages_repo
        .expect_append()
        .once()
        .in_sequence(&mut seq)
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(room),
            predicate::eq(ClientRoomEventType::MessagesAppended {
                message_ids: vec!["message-id".into()],
            }),
        )
        .return_once(|_, _| ());

    deps.messaging_service
        .expect_send_read_receipt()
        .once()
        .in_sequence(&mut seq)
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Received(
                Message::default()
                    .set_type(MessageType::Chat)
                    .set_id("message-id".into())
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
    let mut seq = Sequence::new();

    let room = Room::group(room_id!("room@conference.prose.org"));

    let sent_message = prose_xmpp::stanza::Message::new()
        .set_type(MessageType::Groupchat)
        .set_id("message-id".into())
        .set_from(full!("from@prose.org/res"))
        .set_to(bare!("room@conference.prose.org"))
        .set_body("Hello World")
        .set_chat_state(Some(ChatState::Active))
        .set_markable();

    let expected_saved_message = MessageLike {
        id: "message-id".into(),
        stanza_id: None,
        target: None,
        to: Some(bare!("room@conference.prose.org")),
        from: ParticipantId::User(user_id!("from@prose.org")), // Resource should be dropped
        timestamp: Utc.with_ymd_and_hms(2023, 09, 11, 0, 0, 0).unwrap(),
        payload: MessageLikePayload::Message {
            body: "Hello World".to_string(),
            attachments: vec![],
        },
    };

    deps.time_provider = Arc::new(ConstantTimeProvider::ymd(2023, 09, 11));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .in_sequence(&mut seq)
            .with(predicate::eq(room_id!("room@conference.prose.org")))
            .return_once(|_| Some(room));
    }

    deps.messages_repo
        .expect_append()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(room_id!("room@conference.prose.org")),
            predicate::eq([expected_saved_message]),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(room),
            predicate::eq(ClientRoomEventType::MessagesAppended {
                message_ids: vec!["message-id".into()],
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
async fn test_dispatches_messages_appended_for_new_received_message() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Room::group(room_id!("user@prose.org"));

    deps.sidebar_domain_service
        .expect_insert_item_for_received_message_if_needed()
        .return_once(|_| Box::pin(async { Ok(()) }));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .return_once(|_| Some(room));
    }

    deps.messages_repo
        .expect_contains()
        .return_once(|_| Box::pin(async { Ok(false) }));

    deps.messages_repo
        .expect_append()
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room),
            predicate::eq(ClientRoomEventType::MessagesAppended {
                message_ids: vec!["message-id".into()],
            }),
        )
        .return_once(|_, _| ());

    deps.messaging_service
        .expect_send_read_receipt()
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Received(
                Message::default()
                    .set_id("message-id".into())
                    .set_from(jid!("user@prose.org"))
                    .set_body("Hello World"),
            ),
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_dispatches_messages_updated_for_existing_received_message() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Room::group(room_id!("user@prose.org"));

    deps.sidebar_domain_service
        .expect_insert_item_for_received_message_if_needed()
        .return_once(|_| Box::pin(async { Ok(()) }));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .return_once(|_| Some(room));
    }

    deps.messages_repo
        .expect_contains()
        .return_once(|_| Box::pin(async { Ok(true) }));

    deps.messages_repo
        .expect_append()
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room),
            predicate::eq(ClientRoomEventType::MessagesUpdated {
                message_ids: vec!["message-id".into()],
            }),
        )
        .return_once(|_, _| ());

    deps.messaging_service
        .expect_send_read_receipt()
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Received(
                Message::default()
                    .set_id("message-id".into())
                    .set_from(jid!("user@prose.org"))
                    .set_body("Hello World"),
            ),
        }))
        .await?;

    Ok(())
}
