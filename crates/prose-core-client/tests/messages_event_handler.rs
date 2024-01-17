// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use mockall::{predicate, Sequence};
use xmpp_parsers::message::MessageType;

use prose_core_client::app::event_handlers::{
    MessageEvent, MessageEventType, MessagesEventHandler, ServerEvent, ServerEventHandler,
};
use prose_core_client::domain::rooms::models::RoomInternals;
use prose_core_client::domain::shared::models::{OccupantId, RoomId, UserEndpointId, UserId};
use prose_core_client::dtos::Availability;
use prose_core_client::test::MockAppDependencies;
use prose_core_client::{occupant_id, room_id, user_id, ClientRoomEventType};
use prose_xmpp::jid;
use prose_xmpp::stanza::Message;

#[tokio::test]
async fn test_receiving_message_adds_item_to_sidebar_if_needed() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    let mut seq = Sequence::new();

    let room = Arc::new(
        RoomInternals::group(room_id!("group@conference.prose.org")).with_name("Group Name"),
    );

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

    let room = Arc::new(RoomInternals::direct_message(
        user_id!("jane.doe@prose.org"),
        Availability::Unavailable,
    ));

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
