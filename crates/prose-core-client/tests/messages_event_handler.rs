// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use mockall::predicate;
use xmpp_parsers::message::MessageType;

use prose_core_client::app::event_handlers::{MessagesEventHandler, XMPPEvent, XMPPEventHandler};
use prose_core_client::domain::rooms::models::RoomInternals;
use prose_core_client::domain::rooms::services::{CreateOrEnterRoomRequest, CreateRoomType};
use prose_core_client::domain::shared::models::RoomJid;
use prose_core_client::test::{mock_data, MockAppDependencies};
use prose_core_client::{room, RoomEventType};
use prose_xmpp::mods::chat;
use prose_xmpp::stanza::Message;
use prose_xmpp::{bare, jid};

#[tokio::test]
async fn test_receiving_message_adds_item_to_sidebar_if_needed() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room =
        Arc::new(RoomInternals::group(room!("group@conference.prose.org")).with_name("Group Name"));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room!("group@conference.prose.org")))
            .return_once(|_| Some(room));
    }

    deps.sidebar_domain_service
        .expect_insert_item_for_received_message_if_needed()
        .once()
        .with(predicate::eq(room!("group@conference.prose.org")))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.messages_repo
        .expect_append()
        .once()
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.messaging_service
        .expect_send_read_receipt()
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room),
            predicate::eq(RoomEventType::MessagesAppended {
                message_ids: vec!["message-id".into()],
            }),
        )
        .return_once(|_, _| ());

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(XMPPEvent::Chat(chat::Event::Message(
            Message::default()
                .set_id("message-id".into())
                .set_from(jid!("group@conference.prose.org/jane.doe"))
                .set_body("Hello World"),
        )))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_receiving_message_from_new_contact_creates_room() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(RoomInternals::direct_message(bare!("jane.doe@prose.org")));

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(room!("jane.doe@prose.org")))
        .return_once(|_| None);

    deps.sidebar_domain_service
        .expect_insert_item_by_creating_or_joining_room()
        .once()
        .with(predicate::eq(CreateOrEnterRoomRequest::Create {
            service: mock_data::muc_service(),
            room_type: CreateRoomType::DirectMessage {
                participant: bare!("jane.doe@prose.org"),
            },
        }))
        .return_once(|_| Box::pin(async { Ok(room!("jane.doe@prose.org")) }));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room!("jane.doe@prose.org")))
            .return_once(|_| Some(room));
    }

    deps.sidebar_domain_service
        .expect_insert_item_for_received_message_if_needed()
        .once()
        .with(predicate::eq(room!("jane.doe@prose.org")))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.messages_repo
        .expect_append()
        .once()
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.messaging_service
        .expect_send_read_receipt()
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room),
            predicate::eq(RoomEventType::MessagesAppended {
                message_ids: vec!["message-id".into()],
            }),
        )
        .return_once(|_, _| ());

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(XMPPEvent::Chat(chat::Event::Message(
            Message::default()
                .set_type(MessageType::Chat)
                .set_id("message-id".into())
                .set_from(jid!("jane.doe@prose.org"))
                .set_body("Hello World"),
        )))
        .await?;

    Ok(())
}
