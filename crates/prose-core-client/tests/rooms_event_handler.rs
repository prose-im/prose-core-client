// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use chrono::{TimeZone, Utc};
use mockall::{predicate, Sequence};

use prose_core_client::app::event_handlers::{
    OccupantEvent, OccupantEventType, RoomEvent, RoomEventType, RoomsEventHandler, ServerEvent,
    ServerEventHandler, UserStatusEvent, UserStatusEventType,
};
use prose_core_client::domain::connection::models::ConnectionProperties;
use prose_core_client::domain::rooms::models::{
    ComposeState, RoomAffiliation, RoomInternals, RoomSidebarState,
};
use prose_core_client::domain::rooms::services::{
    CreateOrEnterRoomRequest, JoinRoomBehavior, RoomFactory,
};
use prose_core_client::domain::shared::models::{
    OccupantId, RoomId, UserId, UserOrResourceId, UserResourceId,
};
use prose_core_client::domain::user_info::models::Presence;
use prose_core_client::dtos::{Availability, Participant, ParticipantInfo, UserBasicInfo};
use prose_core_client::test::{
    ConstantTimeProvider, MockAppDependencies, MockRoomFactoryDependencies,
};
use prose_core_client::{
    occupant_id, room_id, user_id, user_resource_id, ClientEvent, ClientRoomEventType,
};

#[tokio::test]
async fn test_adds_participant() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(RoomInternals::group(room_id!("room@conference.prose.org")));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .times(2)
            .with(predicate::eq(room_id!("room@conference.prose.org")))
            .returning(move |_| Some(room.clone()));
    }

    deps.user_profile_repo
        .expect_get_display_name()
        .once()
        .with(predicate::eq(user_id!("real-jid@prose.org")))
        .return_once(|_| Box::pin(async { Ok(Some("George Washington".to_string())) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room.clone()),
            predicate::eq(ClientRoomEventType::ParticipantsChanged),
        )
        .return_once(|_, _| ());

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(ServerEvent::UserStatus(UserStatusEvent {
            user_id: occupant_id!("room@conference.prose.org/nick").into(),
            r#type: UserStatusEventType::AvailabilityChanged {
                availability: Availability::Available,
                priority: 0,
            },
        }))
        .await?;
    event_handler
        .handle_event(ServerEvent::Occupant(OccupantEvent {
            occupant_id: occupant_id!("room@conference.prose.org/nick"),
            anon_occupant_id: None,
            real_id: Some(user_id!("real-jid@prose.org")),
            is_self: false,
            r#type: OccupantEventType::AffiliationChanged {
                affiliation: RoomAffiliation::Member,
            },
        }))
        .await?;

    assert_eq!(room.participants().len(), 1);

    let occupant = room
        .participants()
        .get(&occupant_id!("room@conference.prose.org/nick").into())
        .unwrap()
        .clone();

    assert_eq!(
        occupant,
        Participant {
            real_id: Some(user_id!("real-jid@prose.org")),
            name: Some("George Washington".to_string()),
            is_self: false,
            affiliation: RoomAffiliation::Member,
            availability: Availability::Available,
            compose_state: ComposeState::Idle,
            compose_state_updated: Default::default(),
            anon_occupant_id: None,
        }
    );

    Ok(())
}

#[tokio::test]
async fn test_adds_invited_participant() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(RoomInternals::private_channel(room_id!(
        "room@conference.prose.org"
    )));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room_id!("room@conference.prose.org")))
            .return_once(move |_| Some(room.clone()));
    }

    deps.user_profile_repo
        .expect_get_display_name()
        .once()
        .with(predicate::eq(user_id!("user@prose.org")))
        .return_once(|_| Box::pin(async { Ok(Some("John Doe".to_string())) }));

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room.clone()),
            predicate::eq(ClientRoomEventType::ParticipantsChanged),
        )
        .return_once(|_, _| ());

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(ServerEvent::Room(RoomEvent {
            room_id: room_id!("room@conference.prose.org"),
            r#type: RoomEventType::UserAdded {
                user_id: user_id!("user@prose.org"),
                affiliation: RoomAffiliation::Member,
                reason: None,
            },
        }))
        .await?;

    assert_eq!(
        room.participants()
            .iter()
            .map(ParticipantInfo::from)
            .collect::<Vec<_>>(),
        vec![ParticipantInfo {
            id: Some(user_id!("user@prose.org")),
            name: "John Doe".to_string(),
            is_self: false,
            availability: Availability::Unavailable,
            affiliation: RoomAffiliation::Member,
        }]
    );

    Ok(())
}

#[tokio::test]
async fn test_handles_disconnected_participant() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(
        RoomInternals::private_channel(room_id!("room@conference.prose.org")).with_participants(
            vec![(
                occupant_id!("room@conference.prose.org/a"),
                Participant {
                    real_id: None,
                    anon_occupant_id: None,
                    name: None,
                    is_self: false,
                    affiliation: RoomAffiliation::Admin,
                    availability: Availability::Available,
                    compose_state: ComposeState::Composing,
                    compose_state_updated: Default::default(),
                },
            )],
        ),
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .times(2)
            .with(predicate::eq(room_id!("room@conference.prose.org")))
            .returning(move |_| Some(room.clone()));
    }

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room.clone()),
            predicate::eq(ClientRoomEventType::ParticipantsChanged),
        )
        .return_once(|_, _| ());

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(ServerEvent::UserStatus(UserStatusEvent {
            user_id: occupant_id!("room@conference.prose.org/a").into(),
            r#type: UserStatusEventType::AvailabilityChanged {
                availability: Availability::Unavailable,
                priority: 0,
            },
        }))
        .await?;
    event_handler
        .handle_event(ServerEvent::Occupant(OccupantEvent {
            occupant_id: occupant_id!("room@conference.prose.org/a"),
            anon_occupant_id: None,
            real_id: None,
            is_self: false,
            r#type: OccupantEventType::AffiliationChanged {
                affiliation: RoomAffiliation::Member,
            },
        }))
        .await?;

    assert_eq!(
        room.participants().iter().cloned().collect::<Vec<_>>(),
        vec![Participant {
            real_id: None,
            anon_occupant_id: None,
            name: None,
            is_self: false,
            affiliation: RoomAffiliation::Member,
            availability: Availability::Unavailable,
            compose_state: ComposeState::Idle,
            compose_state_updated: Default::default(),
        }]
    );

    Ok(())
}

#[tokio::test]
async fn test_handles_kicked_user() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(
        RoomInternals::group(room_id!("room@conference.prose.org")).with_participants([(
            occupant_id!("room@conference.prose.org/nickname"),
            Participant::owner().set_real_id(&user_id!("nickname@prose.org")),
        )]),
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room_id!("room@conference.prose.org")))
            .returning(move |_| Some(room.clone()));
    }

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room.clone()),
            predicate::eq(ClientRoomEventType::ParticipantsChanged),
        )
        .return_once(|_, _| ());

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    assert_eq!(room.participants().len(), 1);

    event_handler
        .handle_event(ServerEvent::Occupant(OccupantEvent {
            occupant_id: occupant_id!("room@conference.prose.org/nickname"),
            anon_occupant_id: None,
            real_id: None,
            is_self: false,
            r#type: OccupantEventType::PermanentlyRemoved,
        }))
        .await?;

    assert_eq!(room.participants().len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_handles_kicked_self() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(RoomInternals::group(room_id!("room@conference.prose.org")));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room_id!("room@conference.prose.org")))
            .returning(move |_| Some(room.clone()));
    }

    deps.sidebar_domain_service
        .expect_handle_removal_from_room()
        .once()
        .with(
            predicate::eq(room_id!("room@conference.prose.org")),
            predicate::eq(true),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(ServerEvent::Occupant(OccupantEvent {
            occupant_id: occupant_id!("room@conference.prose.org/nickname"),
            anon_occupant_id: None,
            real_id: None,
            is_self: true,
            r#type: OccupantEventType::PermanentlyRemoved,
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_destroyed_room() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.sidebar_domain_service
        .expect_handle_destroyed_room()
        .once()
        .with(
            predicate::eq(room_id!("group@prose.org")),
            predicate::eq(Some(room_id!("private-channel@prose.org"))),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(ServerEvent::Room(RoomEvent {
            room_id: room_id!("group@prose.org"),
            r#type: RoomEventType::Destroyed {
                replacement: Some(room_id!("private-channel@prose.org")),
            },
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_compose_state_for_muc_room() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(
        RoomInternals::group(room_id!("room@conference.prose.org")).with_participants([(
            occupant_id!("room@conference.prose.org/nickname"),
            Participant::owner()
                .set_real_id(&user_id!("nickname@prose.org"))
                .set_name("Janice Doe"),
        )]),
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room_id!("room@conference.prose.org")))
            .return_once(move |_| Some(room.clone()));
    }
    deps.time_provider = Arc::new(ConstantTimeProvider::ymd(2023, 01, 04));
    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room.clone()),
            predicate::eq(ClientRoomEventType::ComposingUsersChanged),
        )
        .return_once(|_, _| ());

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(ServerEvent::UserStatus(UserStatusEvent {
            user_id: occupant_id!("room@conference.prose.org/nickname").into(),
            r#type: UserStatusEventType::ComposeStateChanged {
                state: ComposeState::Composing,
            },
        }))
        .await?;

    let occupant = room
        .participants()
        .get(&occupant_id!("room@conference.prose.org/nickname").into())
        .unwrap()
        .clone();

    assert_eq!(occupant.compose_state, ComposeState::Composing);
    assert_eq!(
        occupant.compose_state_updated,
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
            id: user_id!("nickname@prose.org")
        }]
    );

    time_provider.set_ymd_hms(2023, 01, 04, 00, 00, 31);
    assert!(room.load_composing_users().await?.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_handles_compose_state_for_direct_message_room() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(RoomInternals::for_direct_message(
        &user_id!("contact@prose.org"),
        "Janice Doe",
        Availability::Unavailable,
        RoomSidebarState::InSidebar,
    ));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room_id!("contact@prose.org")))
            .return_once(move |_| Some(room.clone()));
    }
    deps.time_provider = Arc::new(ConstantTimeProvider::ymd(2023, 01, 04));
    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room.clone()),
            predicate::eq(ClientRoomEventType::ComposingUsersChanged),
        )
        .return_once(|_, _| ());

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(ServerEvent::UserStatus(UserStatusEvent {
            user_id: user_resource_id!("contact@prose.org/resource").into(),
            r#type: UserStatusEventType::ComposeStateChanged {
                state: ComposeState::Composing,
            },
        }))
        .await?;

    let occupant = room
        .participants()
        .get(&user_id!("contact@prose.org").into())
        .unwrap()
        .clone();

    assert_eq!(occupant.compose_state, ComposeState::Composing);
    assert_eq!(
        occupant.compose_state_updated,
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
            id: user_id!("contact@prose.org")
        }]
    );

    time_provider.set_ymd_hms(2023, 01, 04, 00, 00, 31);
    assert!(room.load_composing_users().await?.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_handles_invite() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.sidebar_domain_service
        .expect_insert_item_by_creating_or_joining_room()
        .once()
        .with(predicate::eq(CreateOrEnterRoomRequest::JoinRoom {
            room_id: room_id!("group@conference.prose.org"),
            password: None,
            behavior: JoinRoomBehavior::system_initiated(),
        }))
        .return_once(|_| Box::pin(async move { Ok(room_id!("group@conference.prose.org")) }));

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(ServerEvent::Room(RoomEvent {
            room_id: room_id!("group@conference.prose.org"),
            r#type: RoomEventType::ReceivedInvitation {
                sender: user_resource_id!("user@prose.org/res"),
                password: None,
            },
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_presence() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(RoomInternals::for_direct_message(
        &user_id!("sender@prose.org"),
        "Janice Doe",
        Availability::Unavailable,
        RoomSidebarState::InSidebar,
    ));

    let room = room.clone();
    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(room_id!("sender@prose.org")))
        .return_once(move |_| Some(room.clone()));

    deps.user_info_repo
        .expect_set_user_presence()
        .once()
        .with(
            predicate::eq(UserOrResourceId::from(user_resource_id!(
                "sender@prose.org/resource"
            ))),
            predicate::eq(Presence {
                priority: 1,
                availability: Availability::Available,
                status: None,
            }),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::ContactChanged {
            id: user_id!("sender@prose.org"),
        }))
        .return_once(|_| ());

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(ServerEvent::UserStatus(UserStatusEvent {
            user_id: user_resource_id!("sender@prose.org/resource").into(),
            r#type: UserStatusEventType::AvailabilityChanged {
                availability: Availability::Available,
                priority: 1,
            },
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_contact_presence_with_no_room() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(room_id!("sender@prose.org")))
        .return_once(move |_| None);

    deps.user_info_repo
        .expect_set_user_presence()
        .once()
        .with(
            predicate::eq(UserOrResourceId::from(user_resource_id!(
                "sender@prose.org/resource"
            ))),
            predicate::eq(Presence {
                priority: 1,
                availability: Availability::Available,
                status: None,
            }),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::ContactChanged {
            id: user_id!("sender@prose.org"),
        }))
        .return_once(|_| ());

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(ServerEvent::UserStatus(UserStatusEvent {
            user_id: user_resource_id!("sender@prose.org/resource").into(),
            r#type: UserStatusEventType::AvailabilityChanged {
                availability: Availability::Available,
                priority: 1,
            },
        }))
        .await?;

    Ok(())
}

#[tokio::test]
/// Test that UserStateEventHandler does not send an event when a self-presence is received and
/// that the event is consumed, i.e. cannot be forwarded to other handlers.
async fn test_swallows_self_presence() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.ctx.set_connection_properties(ConnectionProperties {
        connected_jid: user_resource_id!("hello@prose.org/res"),
        server_features: Default::default(),
    });

    let room = Arc::new(RoomInternals::for_direct_message(
        &user_id!("hello@prose.org"),
        "Janice Doe",
        Availability::Unavailable,
        RoomSidebarState::InSidebar,
    ));

    let room = room.clone();
    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(room_id!("hello@prose.org")))
        .return_once(move |_| Some(room.clone()));

    deps.user_info_repo
        .expect_set_user_presence()
        .once()
        .with(
            predicate::eq(UserOrResourceId::from(user_id!("hello@prose.org"))),
            predicate::eq(Presence {
                availability: Availability::Available,
                ..Default::default()
            }),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    let event_handler = RoomsEventHandler::from(&deps.into_deps());
    assert!(event_handler
        .handle_event(ServerEvent::UserStatus(UserStatusEvent {
            user_id: user_id!("hello@prose.org").into(),
            r#type: UserStatusEventType::AvailabilityChanged {
                availability: Availability::Available,
                priority: 0
            }
        }))
        .await?
        .is_none());

    Ok(())
}

#[tokio::test]
async fn test_room_config_changed() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.sidebar_domain_service
        .expect_handle_changed_room_config()
        .once()
        .with(predicate::eq(room_id!("room@conference.prose.org")))
        .return_once(|_| Box::pin(async { Ok(()) }));

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(ServerEvent::Room(RoomEvent {
            room_id: room_id!("room@conference.prose.org"),
            r#type: RoomEventType::RoomConfigChanged,
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_room_topic_changed() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    let mut seq = Sequence::new();

    let room = Arc::new(
        RoomInternals::group(room_id!("room@conference.prose.org")).with_topic(Some("Old Topic")),
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .in_sequence(&mut seq)
            .with(predicate::eq(room_id!("room@conference.prose.org")))
            .returning(move |_| Some(room.clone()));
    }

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .in_sequence(&mut seq)
            .with(predicate::eq(room_id!("room@conference.prose.org")))
            .returning(move |_| Some(room.clone()));
    }

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(room.clone()),
            predicate::eq(ClientRoomEventType::AttributesChanged),
        )
        .return_once(|_, _| ());

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    // Should not generate an event since the topic didn't actually change
    event_handler
        .handle_event(ServerEvent::Room(RoomEvent {
            room_id: room_id!("room@conference.prose.org"),
            r#type: RoomEventType::RoomTopicChanged {
                new_topic: Some("Old Topic".to_string()),
            },
        }))
        .await?;

    // Should fire an event
    event_handler
        .handle_event(ServerEvent::Room(RoomEvent {
            room_id: room_id!("room@conference.prose.org"),
            r#type: RoomEventType::RoomTopicChanged {
                new_topic: Some("New Topic".to_string()),
            },
        }))
        .await?;

    assert_eq!(room.topic(), Some("New Topic".to_string()));

    Ok(())
}
