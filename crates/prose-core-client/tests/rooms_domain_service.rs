// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{Arc, OnceLock};

use anyhow::Result;
use mockall::{predicate, Sequence};
use parking_lot::Mutex;

use prose_core_client::app::event_handlers::{
    OccupantEvent, OccupantEventType, RoomsEventHandler, ServerEvent, ServerEventHandler,
    UserStatusEvent, UserStatusEventType,
};
use prose_core_client::domain::connection::models::ConnectionProperties;
use prose_core_client::domain::rooms::models::{
    RegisteredMember, RoomAffiliation, RoomConfig, RoomError, RoomInternals, RoomSessionInfo,
    RoomSessionMember, RoomSpec,
};
use prose_core_client::domain::rooms::services::impls::RoomsDomainService;
use prose_core_client::domain::rooms::services::{
    CreateOrEnterRoomRequest, CreateRoomType, RoomsDomainService as RoomsDomainServiceTrait,
};
use prose_core_client::domain::shared::models::{OccupantId, RoomId, RoomType, UserResourceId};
use prose_core_client::dtos::{
    Availability, Participant, ParticipantInfo, PublicRoomInfo, UserId, UserProfile,
};
use prose_core_client::test::{mock_data, MockAppDependencies, MockRoomsDomainServiceDependencies};
use prose_core_client::{occupant_id, room_id, user_id, user_resource_id};
use prose_xmpp::test::IncrementingIDProvider;

#[tokio::test]
async fn test_joins_room() -> Result<()> {
    // This test simulates the process of joining a room. It also simulates the received presence
    // events from online occupants and sends these to the RoomsEventHandler to make sure that
    // we don't have duplicate participants afterwards (registered members & occupants).

    let mut deps = MockRoomsDomainServiceDependencies::default();
    let mut seq = Sequence::new();
    let event_handler = Arc::new(OnceLock::<RoomsEventHandler>::new());

    let room = Arc::new(Mutex::new(Arc::new(RoomInternals::pending(
        &room_id!("room@conf.prose.org"),
        "user1#dXNlcjFAcHJvc2Uub3Jn",
    ))));

    deps.ctx.set_connection_properties(ConnectionProperties {
        connected_jid: user_resource_id!("user1@prose.org/res"),
        server_features: Default::default(),
    });

    deps.connected_rooms_repo
        .expect_set()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(room.lock().clone()))
        .return_once(|_| Ok(()));

    let events = vec![
        ServerEvent::UserStatus(UserStatusEvent {
            user_id: occupant_id!("room@conf.prose.org/user1#dXNlcjFAcHJvc2Uub3Jn").into(),
            r#type: UserStatusEventType::AvailabilityChanged {
                availability: Availability::Available,
                priority: 0,
            },
        }),
        ServerEvent::Occupant(OccupantEvent {
            occupant_id: occupant_id!("room@conf.prose.org/user1#dXNlcjFAcHJvc2Uub3Jn"),
            anon_occupant_id: None,
            real_id: Some(user_id!("user1@prose.org")),
            is_self: true,
            r#type: OccupantEventType::AffiliationChanged {
                affiliation: RoomAffiliation::Owner,
            },
        }),
        ServerEvent::UserStatus(UserStatusEvent {
            user_id: occupant_id!("room@conf.prose.org/user2#dXNlcjJAcHJvc2Uub3Jn").into(),
            r#type: UserStatusEventType::AvailabilityChanged {
                availability: Availability::Available,
                priority: 0,
            },
        }),
        ServerEvent::Occupant(OccupantEvent {
            occupant_id: occupant_id!("room@conf.prose.org/user2#dXNlcjJAcHJvc2Uub3Jn"),
            anon_occupant_id: None,
            real_id: Some(user_id!("user2@prose.org")),
            is_self: false,
            r#type: OccupantEventType::AffiliationChanged {
                affiliation: RoomAffiliation::Member,
            },
        }),
    ];

    {
        let event_handler = event_handler.clone();
        deps.room_management_service
            .expect_join_room()
            .once()
            .in_sequence(&mut seq)
            .with(
                predicate::eq(occupant_id!(
                    "room@conf.prose.org/user1#dXNlcjFAcHJvc2Uub3Jn"
                )),
                predicate::always(),
            )
            .return_once(|_, _| {
                Box::pin(async move {
                    let event_handler = event_handler.get().unwrap();

                    for event in events {
                        event_handler
                            .handle_event(event)
                            .await
                            .expect("Unexpected error");
                    }

                    Ok(RoomSessionInfo {
                        room_id: room_id!("room@conf.prose.org"),
                        config: RoomConfig {
                            room_name: Some("Room Name".to_string()),
                            room_description: None,
                            room_type: RoomType::PrivateChannel,
                        },
                        user_nickname: "user#dXNlcjFAcHJvc2Uub3Jn".to_string(),
                        members: vec![
                            RoomSessionMember {
                                id: user_id!("user1@prose.org"),
                                affiliation: RoomAffiliation::Owner,
                            },
                            RoomSessionMember {
                                id: user_id!("user2@prose.org"),
                                affiliation: RoomAffiliation::Member,
                            },
                            RoomSessionMember {
                                id: user_id!("user3@prose.org"),
                                affiliation: RoomAffiliation::Member,
                            },
                        ],
                        room_has_been_created: false,
                    })
                })
            });
    }

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .times(6)
            .with(predicate::eq(room_id!("room@conf.prose.org")))
            .returning(move |_| Some(room.lock().clone()));
    }

    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .times(3)
        .returning(|_, _| ());

    deps.user_profile_repo
        .expect_get_display_name()
        .times(6)
        .with(predicate::in_iter([
            user_id!("user1@prose.org"),
            user_id!("user2@prose.org"),
            user_id!("user3@prose.org"),
        ]))
        .returning(|user_id| {
            let username = user_id.formatted_username();
            Box::pin(async move { Ok(Some(username)) })
        });

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_update()
            .once()
            .in_sequence(&mut seq)
            .with(
                predicate::eq(room_id!("room@conf.prose.org")),
                predicate::always(),
            )
            .return_once(move |_, handler| {
                let updated_room = Arc::new(handler(room.lock().clone()));
                *room.lock() = updated_room.clone();
                Some(updated_room)
            });
    }

    let rooms_deps = deps.into_deps();
    let service = Arc::new(RoomsDomainService::from(rooms_deps.clone()));

    let mut deps = MockAppDependencies::default().into_deps();
    deps.rooms_domain_service = service.clone();
    deps.client_event_dispatcher = rooms_deps.client_event_dispatcher.clone();
    deps.connected_rooms_repo = rooms_deps.connected_rooms_repo.clone();
    deps.ctx = rooms_deps.ctx.clone();
    deps.id_provider = rooms_deps.id_provider.clone();
    deps.room_attributes_service = rooms_deps.room_attributes_service.clone();
    deps.room_management_service = rooms_deps.room_management_service.clone();
    deps.room_participation_service = rooms_deps.room_participation_service.clone();
    deps.user_profile_repo = rooms_deps.user_profile_repo.clone();

    event_handler
        .set(RoomsEventHandler::from(&deps))
        .map_err(|_| ())
        .unwrap();

    service
        .create_or_join_room(CreateOrEnterRoomRequest::JoinRoom {
            room_jid: room_id!("room@conf.prose.org"),
            password: None,
        })
        .await?;

    let mut participants = room
        .lock()
        .participants()
        .iter()
        .map(ParticipantInfo::from)
        .collect::<Vec<_>>();
    participants.sort_by_key(|p| p.name.clone());

    assert_eq!(
        participants,
        vec![
            ParticipantInfo {
                id: Some(user_id!("user1@prose.org")),
                name: "User1".to_string(),
                availability: Availability::Available,
                affiliation: RoomAffiliation::Owner
            },
            ParticipantInfo {
                id: Some(user_id!("user2@prose.org")),
                name: "User2".to_string(),
                availability: Availability::Available,
                affiliation: RoomAffiliation::Member
            },
            ParticipantInfo {
                id: Some(user_id!("user3@prose.org")),
                name: "User3".to_string(),
                availability: Availability::Unavailable,
                affiliation: RoomAffiliation::Member
            }
        ]
    );

    let events = vec![
        ServerEvent::UserStatus(UserStatusEvent {
            user_id: occupant_id!("room@conf.prose.org/user3#dXNlcjNAcHJvc2Uub3Jn").into(),
            r#type: UserStatusEventType::AvailabilityChanged {
                availability: Availability::Available,
                priority: 0,
            },
        }),
        ServerEvent::Occupant(OccupantEvent {
            occupant_id: occupant_id!("room@conf.prose.org/user3#dXNlcjNAcHJvc2Uub3Jn"),
            anon_occupant_id: None,
            real_id: Some(user_id!("user3@prose.org")),
            is_self: false,
            r#type: OccupantEventType::AffiliationChanged {
                affiliation: RoomAffiliation::Member,
            },
        }),
    ];

    let event_handler = event_handler.get().unwrap();

    for event in events {
        event_handler
            .handle_event(event)
            .await
            .expect("Unexpected error");
    }

    let mut participants = room
        .lock()
        .participants()
        .iter()
        .map(ParticipantInfo::from)
        .collect::<Vec<_>>();
    participants.sort_by_key(|p| p.name.clone());

    assert_eq!(
        participants,
        vec![
            ParticipantInfo {
                id: Some(user_id!("user1@prose.org")),
                name: "User1".to_string(),
                availability: Availability::Available,
                affiliation: RoomAffiliation::Owner
            },
            ParticipantInfo {
                id: Some(user_id!("user2@prose.org")),
                name: "User2".to_string(),
                availability: Availability::Available,
                affiliation: RoomAffiliation::Member
            },
            ParticipantInfo {
                id: Some(user_id!("user3@prose.org")),
                name: "User3".to_string(),
                availability: Availability::Available,
                affiliation: RoomAffiliation::Member
            }
        ]
    );

    Ok(())
}

#[tokio::test]
async fn test_throws_conflict_error_if_room_exists() -> Result<()> {
    let mut deps = MockRoomsDomainServiceDependencies::default();

    deps.room_management_service
        .expect_load_public_rooms()
        .once()
        .return_once(|_| {
            Box::pin(async {
                Ok(vec![PublicRoomInfo {
                    jid: room_id!("room@conference.prose.org"),
                    name: Some("new channel".to_string()),
                }])
            })
        });

    let service = RoomsDomainService::from(deps.into_deps());
    let result = service
        .create_or_join_room(CreateOrEnterRoomRequest::Create {
            service: mock_data::muc_service(),
            room_type: CreateRoomType::PublicChannel {
                name: "New Channel".to_string(),
            },
        })
        .await;

    let Err(RoomError::PublicChannelNameConflict) = result else {
        panic!("Expected RoomError::PublicChannelNameConflict")
    };

    Ok(())
}

#[tokio::test]
async fn test_creates_group() -> Result<()> {
    let mut deps = MockRoomsDomainServiceDependencies::default();
    deps.id_provider = Arc::new(IncrementingIDProvider::new("hash"));

    // Make sure that the method calls are in the exact order…
    let mut seq = Sequence::new();

    // jane.doe@prose.org + a@prose.org + b@prose.org + c@prose.org
    let group_jid =
        room_id!("org.prose.group.b41be06eda5bac6e7fc5ad069d6cd863c4f329eb@conference.prose.org");
    let occupant_id = group_jid
        .occupant_id_with_nickname("jane.doe-hash-1")
        .unwrap();

    let account_node = mock_data::account_jid().to_user_id().username().to_string();

    {
        let account_node = account_node.clone();
        deps.user_profile_repo
            .expect_get()
            .times(4)
            .in_sequence(&mut seq)
            .returning(move |jid| {
                let jid = jid.clone();
                let account_node = account_node.clone();

                Box::pin(async move {
                    let first_name = match jid.username() {
                        _ if jid.username() == &account_node => "Jane",
                        "a" => "Tick",
                        "b" => "Trick",
                        "c" => "Track",
                        _ => panic!("Unexpected JID"),
                    };

                    let mut user_profile = UserProfile::default();
                    user_profile.first_name = Some(first_name.to_string());

                    Ok(Some(user_profile))
                })
            });
    }

    deps.connected_rooms_repo
        .expect_set()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(Arc::new(RoomInternals::mock_pending_room(
            group_jid.clone(),
            "hash-1",
        ))))
        .return_once(|_| Ok(()));
    {
        let group_jid = group_jid.clone();
        deps.room_management_service
            .expect_create_or_join_room()
            .once()
            .in_sequence(&mut seq)
            .with(
                predicate::eq(occupant_id),
                predicate::eq("Jane, Tick, Track, Trick"),
                predicate::eq(RoomSpec::Group),
            )
            .return_once(|_, _, _| {
                Box::pin(async {
                    Ok(
                        RoomSessionInfo::new_room(group_jid, RoomType::Group).with_members(vec![
                            RoomSessionMember {
                                id: mock_data::account_jid().into_user_id(),
                                affiliation: RoomAffiliation::Owner,
                            },
                        ]),
                    )
                })
            });
    }

    deps.room_management_service
        .expect_set_room_owners()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(group_jid.clone()),
            predicate::eq(vec![
                user_id!("a@prose.org"),
                user_id!("b@prose.org"),
                user_id!("c@prose.org"),
                mock_data::account_jid().into_user_id(),
            ]),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.user_profile_repo
        .expect_get_display_name()
        .times(4)
        .in_sequence(&mut seq)
        .returning(move |jid| {
            let jid = jid.clone();
            let account_node = account_node.clone();

            Box::pin(async move {
                let first_name = match jid.username() {
                    _ if jid.username() == &account_node => "Jane",
                    "a" => "Tick",
                    "b" => "Trick",
                    "c" => "Track",
                    _ => panic!("Unexpected JID"),
                };

                Ok(Some(first_name.to_string()))
            })
        });

    {
        let group_jid = group_jid.clone();
        deps.connected_rooms_repo
            .expect_update()
            .once()
            .in_sequence(&mut seq)
            .with(predicate::eq(group_jid.clone()), predicate::always())
            .return_once(move |_, handler| {
                let room = Arc::new(RoomInternals::mock_pending_room(
                    group_jid.clone(),
                    "hash-1",
                ));

                let room = handler(room.clone());
                let mut members = room.participants().iter().cloned().collect::<Vec<_>>();
                members.sort_by_key(|p| p.real_id.as_ref().unwrap().clone());

                assert_eq!(
                    members,
                    vec![
                        Participant {
                            real_id: Some(user_id!("a@prose.org")),
                            anon_occupant_id: None,
                            name: Some("Tick".to_string()),
                            affiliation: RoomAffiliation::Owner,
                            availability: Default::default(),
                            compose_state: Default::default(),
                            compose_state_updated: Default::default(),
                        },
                        Participant {
                            real_id: Some(user_id!("b@prose.org")),
                            anon_occupant_id: None,
                            name: Some("Trick".to_string()),
                            affiliation: RoomAffiliation::Owner,
                            availability: Default::default(),
                            compose_state: Default::default(),
                            compose_state_updated: Default::default(),
                        },
                        Participant {
                            real_id: Some(user_id!("c@prose.org")),
                            anon_occupant_id: None,
                            name: Some("Track".to_string()),
                            affiliation: RoomAffiliation::Owner,
                            availability: Default::default(),
                            compose_state: Default::default(),
                            compose_state_updated: Default::default(),
                        },
                        Participant {
                            real_id: Some(user_id!("jane.doe@prose.org")),
                            anon_occupant_id: None,
                            name: Some("Jane".to_string()),
                            affiliation: RoomAffiliation::Owner,
                            availability: Default::default(),
                            compose_state: Default::default(),
                            compose_state_updated: Default::default(),
                        }
                    ]
                );

                Some(Arc::new(room))
            });
    }

    deps.room_participation_service
        .expect_invite_users_to_room()
        .once()
        .with(
            predicate::eq(group_jid.clone()),
            predicate::eq(vec![
                user_id!("a@prose.org"),
                user_id!("b@prose.org"),
                user_id!("c@prose.org"),
            ]),
        )
        .returning(|_, _| Box::pin(async { Ok(()) }));

    let service = RoomsDomainService::from(deps.into_deps());
    let result = service
        .create_or_join_room(CreateOrEnterRoomRequest::Create {
            service: mock_data::muc_service(),
            room_type: CreateRoomType::Group {
                participants: vec![
                    user_id!("a@prose.org"),
                    user_id!("b@prose.org"),
                    user_id!("c@prose.org"),
                ],
            },
        })
        .await;

    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_creates_public_room_if_it_does_not_exist() -> Result<()> {
    let mut deps = MockRoomsDomainServiceDependencies::default();

    deps.id_provider = Arc::new(IncrementingIDProvider::new("hash"));

    deps.room_management_service
        .expect_load_public_rooms()
        .once()
        .return_once(|_| {
            Box::pin(async {
                Ok(vec![PublicRoomInfo {
                    jid: room_id!("room@conference.prose.org"),
                    name: Some("Old Channel".to_string()),
                }])
            })
        });

    deps.connected_rooms_repo
        .expect_set()
        .once()
        .with(predicate::eq(Arc::new(RoomInternals::mock_pending_room(
            room_id!("org.prose.channel.hash-1@conference.prose.org"),
            "hash-2",
        ))))
        .return_once(|_| Ok(()));

    deps.room_management_service
        .expect_create_or_join_room()
        .once()
        .return_once(|_, _, _| {
            Box::pin(async {
                Ok(RoomSessionInfo::new_room(
                    room_id!("org.prose.channel.hash-1@conference.prose.org"),
                    RoomType::PublicChannel,
                ))
            })
        });

    deps.connected_rooms_repo
        .expect_update()
        .once()
        .with(
            predicate::eq(room_id!("org.prose.channel.hash-1@conference.prose.org")),
            predicate::always(),
        )
        .return_once(|_, _| {
            Some(Arc::new(RoomInternals::public_channel(room_id!(
                "org.prose.channel.hash-1@conference.prose.org"
            ))))
        });

    let service = RoomsDomainService::from(deps.into_deps());
    let result = service
        .create_or_join_room(CreateOrEnterRoomRequest::Create {
            service: mock_data::muc_service(),
            room_type: CreateRoomType::PublicChannel {
                name: "New Channel".to_string(),
            },
        })
        .await;

    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_converts_group_to_private_channel() -> Result<()> {
    let mut deps = MockRoomsDomainServiceDependencies::default();
    deps.id_provider = Arc::new(IncrementingIDProvider::new("hash"));

    let channel_jid = room_id!("org.prose.channel.hash-1@conf.prose.org");
    let occupant_id = channel_jid
        .occupant_id_with_nickname(&format!("{}-hash-2", mock_data::account_jid().username()))
        .unwrap();

    // Make sure that the method calls are in the exact order…
    let mut seq = Sequence::new();

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(room_id!("group@conf.prose.org")))
        .return_once(|_| {
            Some(Arc::new(
                RoomInternals::group(room_id!("group@conf.prose.org")).with_members(vec![
                    RegisteredMember {
                        user_id: mock_data::account_jid().into_user_id(),
                        name: Some("Jane Doe".to_string()),
                        affiliation: RoomAffiliation::Owner,
                    },
                    RegisteredMember {
                        user_id: user_id!("a@prose.org"),
                        name: Some("Member A".to_string()),
                        affiliation: RoomAffiliation::Owner,
                    },
                    RegisteredMember {
                        user_id: user_id!("b@prose.org"),
                        name: Some("Member B".to_string()),
                        affiliation: RoomAffiliation::Owner,
                    },
                ]),
            ))
        });

    deps.connected_rooms_repo
        .expect_delete()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(room_id!("group@conf.prose.org")))
        .return_once(|_| ());

    deps.connected_rooms_repo
        .expect_set()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(Arc::new(RoomInternals::mock_pending_room(
            channel_jid.clone(),
            "hash-2",
        ))))
        .return_once(|_| Ok(()));

    {
        let channel_jid = channel_jid.clone();
        deps.room_management_service
            .expect_create_or_join_room()
            .once()
            .in_sequence(&mut seq)
            .with(
                predicate::eq(occupant_id),
                predicate::eq("Private Channel"),
                predicate::eq(RoomSpec::PrivateChannel),
            )
            .return_once(|_, _, _| {
                Box::pin(async move {
                    Ok(RoomSessionInfo::new_room(
                        channel_jid.clone(),
                        RoomType::PrivateChannel,
                    ))
                })
            });
    }

    {
        let channel_jid = channel_jid.clone();
        deps.connected_rooms_repo
            .expect_update()
            .once()
            .in_sequence(&mut seq)
            .with(predicate::eq(channel_jid.clone()), predicate::always())
            .return_once(move |_, _| {
                Some(Arc::new(RoomInternals::private_channel(
                    channel_jid.clone(),
                )))
            });
    }

    deps.message_migration_domain_service
        .expect_copy_all_messages_from_room()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(room_id!("group@conf.prose.org")),
            predicate::eq(RoomType::Group),
            predicate::eq(channel_jid.clone()),
            predicate::eq(RoomType::PrivateChannel),
        )
        .return_once(|_, _, _, _| Box::pin(async { Ok(()) }));

    deps.room_participation_service
        .expect_grant_membership()
        .times(2)
        .in_sequence(&mut seq)
        .with(
            predicate::eq(channel_jid.clone()),
            predicate::in_iter(vec![user_id!("a@prose.org"), user_id!("b@prose.org")]),
        )
        .returning(|_, _| Box::pin(async { Ok(()) }));

    deps.room_management_service
        .expect_destroy_room()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(room_id!("group@conf.prose.org")),
            predicate::eq(Some(channel_jid.clone())),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    let service = RoomsDomainService::from(deps.into_deps());

    let room = service
        .reconfigure_room_with_spec(
            &room_id!("group@conf.prose.org"),
            RoomSpec::PrivateChannel,
            "Private Channel",
        )
        .await?;

    assert_eq!(room.r#type, RoomType::PrivateChannel);

    Ok(())
}

#[tokio::test]
async fn test_converts_private_to_public_channel_if_it_does_not_exist() -> Result<()> {
    let mut deps = MockRoomsDomainServiceDependencies::default();

    let room = Arc::new(
        RoomInternals::private_channel(room_id!("channel@conf.prose.org")).with_members(vec![
            RegisteredMember {
                user_id: mock_data::account_jid().into_user_id(),
                name: Some("Jane Doe".to_string()),
                affiliation: RoomAffiliation::Owner,
            },
            RegisteredMember {
                user_id: user_id!("a@prose.org"),
                name: Some("Member A".to_string()),
                affiliation: RoomAffiliation::Owner,
            },
        ]),
    );

    // Make sure that the method calls are in the exact order…
    let mut seq = Sequence::new();

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .in_sequence(&mut seq)
            .with(predicate::eq(room_id!("channel@conf.prose.org")))
            .return_once(|_| Some(room));
    }

    deps.room_management_service
        .expect_load_public_rooms()
        .once()
        .in_sequence(&mut seq)
        .return_once(|_| Box::pin(async { Ok(vec![]) }));

    deps.room_management_service
        .expect_reconfigure_room()
        .with(
            predicate::eq(room_id!("channel@conf.prose.org")),
            predicate::eq(RoomSpec::PublicChannel),
            predicate::eq("Public Channel"),
        )
        .once()
        .in_sequence(&mut seq)
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_update()
            .once()
            .in_sequence(&mut seq)
            .with(
                predicate::eq(room_id!("channel@conf.prose.org")),
                predicate::always(),
            )
            .return_once(|_, handler| Some(Arc::new(handler(room))));
    }

    let service = RoomsDomainService::from(deps.into_deps());

    let room = service
        .reconfigure_room_with_spec(
            &room_id!("channel@conf.prose.org"),
            RoomSpec::PublicChannel,
            "Public Channel",
        )
        .await?;

    assert_eq!(room.r#type, RoomType::PublicChannel);

    Ok(())
}

#[tokio::test]
async fn test_converts_private_to_public_channel_name_conflict() -> Result<()> {
    let mut deps = MockRoomsDomainServiceDependencies::default();

    // Make sure that the method calls are in the exact order…
    let mut seq = Sequence::new();

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(room_id!("channel@conf.prose.org")))
        .return_once(|_| {
            Some(Arc::new(
                RoomInternals::private_channel(room_id!("channel@conf.prose.org")).with_members(
                    vec![
                        RegisteredMember {
                            user_id: mock_data::account_jid().into_user_id(),
                            name: Some("Jane Doe".to_string()),
                            affiliation: RoomAffiliation::Owner,
                        },
                        RegisteredMember {
                            user_id: user_id!("a@prose.org"),
                            name: Some("Member A".to_string()),
                            affiliation: RoomAffiliation::Owner,
                        },
                    ],
                ),
            ))
        });

    deps.room_management_service
        .expect_load_public_rooms()
        .once()
        .in_sequence(&mut seq)
        .return_once(|_| {
            Box::pin(async {
                Ok(vec![PublicRoomInfo {
                    jid: room_id!("room@conference.prose.org"),
                    name: Some("new channel".to_string()),
                }])
            })
        });

    let service = RoomsDomainService::from(deps.into_deps());

    let result = service
        .reconfigure_room_with_spec(
            &room_id!("channel@conf.prose.org"),
            RoomSpec::PublicChannel,
            "New Channel",
        )
        .await;

    let Err(RoomError::PublicChannelNameConflict) = result else {
        panic!(
            "Expected RoomError::PublicChannelNameConflict. Got {:?}",
            result
        )
    };

    Ok(())
}
