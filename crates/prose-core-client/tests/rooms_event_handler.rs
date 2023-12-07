// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use chrono::{TimeZone, Utc};
use mockall::predicate;

use prose_core_client::app::event_handlers::{
    OccupantEvent, OccupantEventType, RoomEvent, RoomEventType, RoomsEventHandler, ServerEvent,
    ServerEventHandler, UserStatusEvent, UserStatusEventType,
};
use prose_core_client::domain::connection::models::ConnectionProperties;
use prose_core_client::domain::rooms::models::{ComposeState, RoomAffiliation, RoomInternals};
use prose_core_client::domain::rooms::services::{CreateOrEnterRoomRequest, RoomFactory};
use prose_core_client::domain::shared::models::{
    OccupantId, RoomId, UserId, UserOrResourceId, UserResourceId,
};
use prose_core_client::domain::user_info::models::Presence;
use prose_core_client::dtos::{Availability, Participant, UserBasicInfo};
use prose_core_client::test::{
    ConstantTimeProvider, MockAppDependencies, MockRoomFactoryDependencies,
};
use prose_core_client::{
    occupant_id, room_id, user_id, user_resource_id, ClientEvent, ClientRoomEventType,
};

#[tokio::test]
async fn test_handles_presence_for_muc_room() -> Result<()> {
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
async fn test_user_entered_room_with_multiple_resources_and_same_nickname() -> Result<()> {
    // <presence xmlns='jabber:client' from="org.prose.public-channel.dev-core#1@groups.prose.org/cram" to="marc@prose.org/prose-QAaMglCf" xml:lang="en">
    //     <x xmlns='vcard-temp:x:update'>
    //     <photo>cdc05cb9c48d5e817a36d462fe0470a0579e570a</photo>
    //     </x><occupant-id xmlns='urn:xmpp:occupant-id:0' id="gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw=" />
    //     <x xmlns='http://jabber.org/protocol/muc#user'>
    // <item affiliation="none" jid="cram@prose.org/prose-GJFeX9yi" role="participant" />
    //     </x>
    //     </presence>
    //
    //     <presence xmlns='jabber:client' from="org.prose.public-channel.dev-core#1@groups.prose.org/cram" to="marc@prose.org/prose-QAaMglCf" xml:lang="en">
    //     <x xmlns='vcard-temp:x:update'>
    //     <photo>cdc05cb9c48d5e817a36d462fe0470a0579e570a</photo>
    //     </x><occupant-id xmlns='urn:xmpp:occupant-id:0' id="gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw=" />
    //     <x xmlns='http://jabber.org/protocol/muc#user'>
    // <item affiliation="none" jid="cram@prose.org/prose-GJFeX9yi" role="participant" />
    //     <item affiliation="none" jid="cram@prose.org/prose-rJF2R8AI" role="participant" />
    //     </x>
    //     </presence>
    //
    //     <presence xmlns='jabber:client' from="org.prose.public-channel.dev-core#1@groups.prose.org/cram" to="marc@prose.org/prose-QAaMglCf" type="unavailable">
    //     <status>Disconnected: closed</status><occupant-id xmlns='urn:xmpp:occupant-id:0' id="gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw=" />
    //     <x xmlns='http://jabber.org/protocol/muc#user'>
    // <item affiliation="none" jid="cram@prose.org/prose-rJF2R8AI" role="none" />
    //     </x>
    //     </presence>
    panic!("Implement me")
}

#[tokio::test]
async fn test_user_entered_room_with_multiple_resources_and_different_nicknames() -> Result<()> {
    // <presence xmlns='jabber:client' from="org.prose.public-channel.dev-core#1@groups.prose.org/cram-t7YjIEMF" to="marc@prose.org/prose-YJlWBFC2" xml:lang="en">
    //     <x xmlns='vcard-temp:x:update'>
    //     <photo>cdc05cb9c48d5e817a36d462fe0470a0579e570a</photo>
    //     </x>
    //     <occupant-id xmlns='urn:xmpp:occupant-id:0' id="gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw=" />
    //     <x xmlns='http://jabber.org/protocol/muc#user'>
    // <item affiliation="none" jid="cram@prose.org/prose-IVO5rzuB" role="participant" />
    //     </x>
    //     </presence>
    //
    //     <presence xmlns='jabber:client' from="org.prose.public-channel.dev-core#1@groups.prose.org/cram-t7YjIEMF" to="marc@prose.org/prose-YJlWBFC2" xml:lang="en">
    //     <x xmlns='vcard-temp:x:update'>
    //     <photo>cdc05cb9c48d5e817a36d462fe0470a0579e570a</photo>
    //     </x>
    //     <occupant-id xmlns='urn:xmpp:occupant-id:0' id="gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw=" />
    //     <x xmlns='http://jabber.org/protocol/muc#user'>
    // <item affiliation="none" jid="cram@prose.org/prose-IVO5rzuB" role="participant" />
    //     </x>
    //     </presence>
    //
    //     <presence xmlns='jabber:client' from="org.prose.public-channel.dev-core#1@groups.prose.org/cram-CnLZs8Id" to="marc@prose.org/prose-YJlWBFC2" type="unavailable">
    //     <status>Disconnected: closed</status>
    //     <occupant-id xmlns='urn:xmpp:occupant-id:0' id="gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw=" />
    //     <x xmlns='http://jabber.org/protocol/muc#user'>
    // <item affiliation="none" jid="cram@prose.org/prose-AJJuBKzn" role="none" />
    //     </x>
    //     </presence>
    //
    //     <presence xmlns='jabber:client' from="org.prose.public-channel.dev-core#1@groups.prose.org/cram-t7YjIEMF" to="marc@prose.org/prose-YJlWBFC2" type="unavailable">
    //     <status>Disconnected: closed</status>
    //     <occupant-id xmlns='urn:xmpp:occupant-id:0' id="gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw=" />
    //     <x xmlns='http://jabber.org/protocol/muc#user'>
    // <item affiliation="none" jid="cram@prose.org/prose-IVO5rzuB" role="none" />
    //     </x>
    //     </presence>
    panic!("Implement me")
}

#[tokio::test]
async fn test_handles_disconnected_participant() -> Result<()> {
    panic!("Implement me")
}

#[tokio::test]
async fn test_handles_added_member() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(
        RoomInternals::private_channel(room_id!("room@conference.prose.org")).with_participants(
            vec![(
                occupant_id!("room@conference.prose.org/a"),
                Participant::owner(),
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

    deps.user_profile_repo
        .expect_get_display_name()
        .once()
        .with(predicate::eq(user_id!("b@prose.org")))
        .return_once(|_| Box::pin(async { Ok(Some("Mike Doe".to_string())) }));

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(ServerEvent::UserStatus(UserStatusEvent {
            user_id: occupant_id!("room@conference.prose.org/b").into(),
            r#type: UserStatusEventType::AvailabilityChanged {
                availability: Availability::Available,
                priority: 0,
            },
        }))
        .await?;
    event_handler
        .handle_event(ServerEvent::Occupant(OccupantEvent {
            occupant_id: occupant_id!("room@conference.prose.org/b"),
            anon_occupant_id: None,
            real_id: Some(user_id!("b@prose.org")),
            is_self: false,
            r#type: OccupantEventType::AffiliationChanged {
                affiliation: RoomAffiliation::Member,
            },
        }))
        .await?;

    let added_participant = room
        .participants()
        .get(&occupant_id!("room@conference.prose.org/b").into())
        .cloned();
    assert_eq!(
        added_participant,
        Some(Participant {
            real_id: Some(user_id!("b@prose.org")),
            anon_occupant_id: None,
            name: Some("Mike Doe".to_string()),
            affiliation: RoomAffiliation::Member,
            availability: Availability::Available,
            compose_state: Default::default(),
            compose_state_updated: Default::default(),
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_handles_kicked_user() -> Result<()> {
    panic!("Implement me")
}

#[tokio::test]
async fn test_handles_disconnected_user() -> Result<()> {
    panic!("Implement me")
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
            room_jid: room_id!("group@conference.prose.org"),
            password: None,
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
