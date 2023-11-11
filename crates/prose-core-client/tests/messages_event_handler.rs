// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use mockall::predicate;

use prose_core_client::app::event_handlers::{MessagesEventHandler, XMPPEvent, XMPPEventHandler};
use prose_core_client::domain::rooms::models::RoomInternals;
use prose_core_client::domain::rooms::services::RoomFactory;
use prose_core_client::domain::sidebar::models::{Bookmark, BookmarkType, SidebarItem};
use prose_core_client::test::{mock_data, MockAppDependencies};
use prose_core_client::{ClientEvent, RoomEventType};
use prose_xmpp::mods::chat;
use prose_xmpp::stanza::Message;
use prose_xmpp::{bare, jid};

#[tokio::test]
async fn test_receiving_message_from_group_adds_group_to_sidebar() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(
        RoomInternals::group(&bare!("group@conference.prose.org")).with_name("Group Name"),
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(bare!("group@conference.prose.org")))
            .return_once(|_| Some(room));
    }

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(bare!("group@conference.prose.org")))
        .return_once(|_| None);

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .with(predicate::eq(Bookmark {
            name: "Group Name".to_string(),
            jid: bare!("group@conference.prose.org"),
            r#type: BookmarkType::Group,
            is_favorite: false,
            in_sidebar: true,
        }))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.sidebar_repo
        .expect_put()
        .once()
        .with(predicate::eq(SidebarItem {
            name: "Group Name".to_string(),
            jid: bare!("group@conference.prose.org"),
            r#type: BookmarkType::Group,
            is_favorite: false,
            error: None,
        }))
        .return_once(|_| ());

    deps.messages_repo
        .expect_append()
        .once()
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.messaging_service
        .expect_send_read_receipt()
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::RoomChanged {
            room: RoomFactory::mock().build(room.clone()),
            r#type: RoomEventType::MessagesAppended {
                message_ids: vec!["message-id".into()],
            },
        }))
        .return_once(|_| ());

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
async fn test_receiving_message_from_contact_adds_contact_to_sidebar() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(RoomInternals::for_direct_message(
        &mock_data::account_jid().into_bare(),
        &bare!("jane.doe@prose.org"),
        "Jane Doe",
    ));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(bare!("jane.doe@prose.org")))
            .return_once(|_| Some(room));
    }

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(bare!("jane.doe@prose.org")))
        .return_once(|_| None);

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .with(predicate::eq(Bookmark {
            name: "Jane Doe".to_string(),
            jid: bare!("jane.doe@prose.org"),
            r#type: BookmarkType::DirectMessage,
            is_favorite: false,
            in_sidebar: true,
        }))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.sidebar_repo
        .expect_put()
        .once()
        .with(predicate::eq(SidebarItem {
            name: "Jane Doe".to_string(),
            jid: bare!("jane.doe@prose.org"),
            r#type: BookmarkType::DirectMessage,
            is_favorite: false,
            error: None,
        }))
        .return_once(|_| ());

    deps.messages_repo
        .expect_append()
        .once()
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.messaging_service
        .expect_send_read_receipt()
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::RoomChanged {
            room: RoomFactory::mock().build(room.clone()),
            r#type: RoomEventType::MessagesAppended {
                message_ids: vec!["message-id".into()],
            },
        }))
        .return_once(|_| ());

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(XMPPEvent::Chat(chat::Event::Message(
            Message::default()
                .set_id("message-id".into())
                .set_from(jid!("jane.doe@prose.org/macOS"))
                .set_body("Hello World"),
        )))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_receiving_message_from_channel_does_not_add_channel_to_sidebar() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(
        RoomInternals::public_channel(&bare!("channel@conference.prose.org"))
            .with_name("Channel Name"),
    );

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(bare!("channel@conference.prose.org")))
        .return_once(|_| Some(room));

    deps.messages_repo
        .expect_append()
        .once()
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.messaging_service
        .expect_send_read_receipt()
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .return_once(|_| ());

    let event_handler = MessagesEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(XMPPEvent::Chat(chat::Event::Message(
            Message::default()
                .set_id("message-id".into())
                .set_from(jid!("channel@conference.prose.org/jane.doe"))
                .set_body("Hello World"),
        )))
        .await?;

    Ok(())
}
