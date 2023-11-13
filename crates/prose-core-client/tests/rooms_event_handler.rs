// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use chrono::{TimeZone, Utc};
use mockall::predicate;
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::muc::user::{Affiliation, Item, Role};
use xmpp_parsers::muc::MucUser;
use xmpp_parsers::presence::Presence;

use prose_core_client::app::event_handlers::{RoomsEventHandler, XMPPEvent, XMPPEventHandler};
use prose_core_client::domain::rooms::models::{RoomInfo, RoomInternals};
use prose_core_client::domain::rooms::services::{
    CreateOrEnterRoomRequest, CreateOrEnterRoomRequestType, RoomFactory,
};
use prose_core_client::domain::shared::models::RoomJid;
use prose_core_client::domain::shared::models::RoomType;
use prose_core_client::dtos::{Occupant, UserBasicInfo};
use prose_core_client::test::{
    mock_data, ConstantTimeProvider, MockAppDependencies, MockRoomFactoryDependencies,
};
use prose_core_client::{room, ClientEvent, RoomEventType};
use prose_xmpp::mods::muc;
use prose_xmpp::stanza::muc::MediatedInvite;
use prose_xmpp::{bare, full, jid, mods};

#[tokio::test]
async fn test_handles_presence_for_muc_room() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(RoomInternals {
        info: RoomInfo {
            jid: room!("room@conference.prose.org"),
            description: None,
            user_jid: mock_data::account_jid().into_bare(),
            user_nickname: "".to_string(),
            members: HashMap::new(),
            room_type: RoomType::Group,
        },
        state: Default::default(),
    });

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room!("room@conference.prose.org")))
            .return_once(move |_| Some(room.clone()));
    }

    deps.user_profile_repo
        .expect_get_display_name()
        .once()
        .with(predicate::eq(bare!("real-jid@prose.org")))
        .return_once(|_| Box::pin(async { Ok(Some("George Washington".to_string())) }));

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(XMPPEvent::Status(mods::status::Event::Presence(
            Presence::available()
                .with_from(full!("room@conference.prose.org/nick"))
                .with_to(mock_data::account_jid())
                .with_payload(MucUser::new().with_items(vec![Item::new(
                        Affiliation::Member,
                        Role::Participant,
                    )
                    .with_jid(full!("real-jid@prose.org/resource"))])),
        )))
        .await?;

    assert_eq!(room.state.read().occupants.len(), 1);

    let occupant = room
        .state
        .read()
        .occupants
        .get(&jid!("room@conference.prose.org/nick"))
        .unwrap()
        .clone();

    assert_eq!(
        occupant,
        Occupant {
            jid: Some(bare!("real-jid@prose.org")),
            name: Some("George Washington".to_string()),
            affiliation: Affiliation::Member,
            chat_state: ChatState::Gone,
            chat_state_updated: Default::default(),
        }
    );

    Ok(())
}

#[tokio::test]
async fn test_handles_chat_state_for_muc_room() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(
        RoomInternals::group(room!("room@conference.prose.org")).with_occupants([(
            jid!("room@conference.prose.org/nickname"),
            Occupant::owner()
                .set_real_jid(&bare!("nickname@prose.org"))
                .set_name("Janice Doe"),
        )]),
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room!("room@conference.prose.org")))
            .return_once(move |_| Some(room.clone()));
    }
    deps.time_provider = Arc::new(ConstantTimeProvider::ymd(2023, 01, 04));
    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::RoomChanged {
            room: RoomFactory::mock().build(room.clone()),
            r#type: RoomEventType::ComposingUsersChanged,
        }))
        .return_once(|_| ());

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(XMPPEvent::Chat(mods::chat::Event::ChatStateChanged {
            from: jid!("room@conference.prose.org/nickname"),
            chat_state: ChatState::Composing,
            message_type: MessageType::Groupchat,
        }))
        .await?;

    let occupant = room
        .state
        .read()
        .occupants
        .get(&jid!("room@conference.prose.org/nickname"))
        .unwrap()
        .clone();

    assert_eq!(occupant.chat_state, ChatState::Composing);
    assert_eq!(
        occupant.chat_state_updated,
        Utc.with_ymd_and_hms(2023, 01, 04, 0, 0, 0).unwrap()
    );

    let time_provider = Arc::new(ConstantTimeProvider::ymd_hms(2023, 01, 04, 00, 00, 20));

    let mut factory_deps = MockRoomFactoryDependencies::default();
    factory_deps.time_provider = time_provider.clone();

    let room_factory = RoomFactory::from(factory_deps);
    let room = room_factory.build(room.clone()).to_generic_room();
    assert_eq!(
        room.load_composing_users().await?,
        vec![UserBasicInfo {
            name: "Janice Doe".to_string(),
            jid: bare!("nickname@prose.org")
        }]
    );

    time_provider.set_ymd_hms(2023, 01, 04, 00, 00, 31);
    assert!(room.load_composing_users().await?.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_handles_chat_state_for_direct_message_room() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(RoomInternals::for_direct_message(
        &mock_data::account_jid().into_bare(),
        &bare!("contact@prose.org"),
        "Janice Doe",
    ));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room!("contact@prose.org")))
            .return_once(move |_| Some(room.clone()));
    }
    deps.time_provider = Arc::new(ConstantTimeProvider::ymd(2023, 01, 04));
    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::RoomChanged {
            room: RoomFactory::mock().build(room.clone()),
            r#type: RoomEventType::ComposingUsersChanged,
        }))
        .return_once(|_| ());

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(XMPPEvent::Chat(mods::chat::Event::ChatStateChanged {
            from: jid!("contact@prose.org/resource"),
            chat_state: ChatState::Composing,
            message_type: MessageType::Chat,
        }))
        .await?;

    let occupant = room
        .state
        .read()
        .occupants
        .get(&jid!("contact@prose.org"))
        .unwrap()
        .clone();

    assert_eq!(occupant.chat_state, ChatState::Composing);
    assert_eq!(
        occupant.chat_state_updated,
        Utc.with_ymd_and_hms(2023, 01, 04, 0, 0, 0).unwrap()
    );

    let time_provider = Arc::new(ConstantTimeProvider::ymd_hms(2023, 01, 04, 00, 00, 20));

    let mut factory_deps = MockRoomFactoryDependencies::default();
    factory_deps.time_provider = time_provider.clone();

    let room_factory = RoomFactory::from(factory_deps);
    let room = room_factory.build(room.clone()).to_generic_room();
    assert_eq!(
        room.load_composing_users().await?,
        vec![UserBasicInfo {
            name: "Janice Doe".to_string(),
            jid: bare!("contact@prose.org")
        }]
    );

    time_provider.set_ymd_hms(2023, 01, 04, 00, 00, 31);
    assert!(room.load_composing_users().await?.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_handles_invite() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.rooms_domain_service
        .expect_create_or_join_room()
        .once()
        .with(predicate::eq(CreateOrEnterRoomRequest {
            r#type: CreateOrEnterRoomRequestType::Join {
                room_jid: room!("group@conference.prose.org"),
                nickname: None,
                password: None,
            },
            save_bookmark: true,
            insert_sidebar_item: true,
            notify_delegate: true,
        }))
        .returning(move |req| {
            let CreateOrEnterRoomRequestType::Join { room_jid, .. } = req.r#type else {
                panic!("Expected CreateOrEnterRoomRequestType::Join")
            };
            let response = Arc::new(RoomInternals::group(room!("group@conference.prose.org")));
            assert_eq!(room_jid, response.info.jid);
            Box::pin(async move { Ok(response) })
        });

    let event_handler = RoomsEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(XMPPEvent::MUC(muc::Event::MediatedInvite {
            from: jid!("group@conference.prose.org"),
            invite: MediatedInvite {
                invites: vec![],
                password: None,
            },
        }))
        .await?;

    Ok(())
}
