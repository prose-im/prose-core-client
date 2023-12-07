// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use mockall::{predicate, Sequence};

use prose_core_client::domain::rooms::models::{
    RegisteredMember, RoomAffiliation, RoomError, RoomInternals, RoomSessionInfo,
    RoomSessionMember, RoomSpec,
};
use prose_core_client::domain::rooms::services::impls::RoomsDomainService;
use prose_core_client::domain::rooms::services::{
    CreateOrEnterRoomRequest, CreateRoomType, RoomsDomainService as RoomsDomainServiceTrait,
};
use prose_core_client::domain::shared::models::{RoomId, RoomType};
use prose_core_client::dtos::{Participant, PublicRoomInfo, UserId, UserProfile};
use prose_core_client::test::{mock_data, MockRoomsDomainServiceDependencies};
use prose_core_client::{room_id, user_id};
use prose_xmpp::test::IncrementingIDProvider;

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
                                nick: None,
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
                        occupant_id: None,
                    },
                    RegisteredMember {
                        user_id: user_id!("a@prose.org"),
                        name: Some("Member A".to_string()),
                        affiliation: RoomAffiliation::Owner,
                        occupant_id: None,
                    },
                    RegisteredMember {
                        user_id: user_id!("b@prose.org"),
                        name: Some("Member B".to_string()),
                        affiliation: RoomAffiliation::Owner,
                        occupant_id: None,
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

    service
        .reconfigure_room_with_spec(
            &room_id!("group@conf.prose.org"),
            RoomSpec::PrivateChannel,
            "Private Channel",
        )
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_converts_private_to_public_channel_if_it_does_not_exist() -> Result<()> {
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
                            occupant_id: None,
                        },
                        RegisteredMember {
                            user_id: user_id!("a@prose.org"),
                            name: Some("Member A".to_string()),
                            affiliation: RoomAffiliation::Owner,
                            occupant_id: None,
                        },
                    ],
                ),
            ))
        });

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

    let service = RoomsDomainService::from(deps.into_deps());

    service
        .reconfigure_room_with_spec(
            &room_id!("channel@conf.prose.org"),
            RoomSpec::PublicChannel,
            "Public Channel",
        )
        .await?;

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
                            occupant_id: None,
                        },
                        RegisteredMember {
                            user_id: user_id!("a@prose.org"),
                            name: Some("Member A".to_string()),
                            affiliation: RoomAffiliation::Owner,
                            occupant_id: None,
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

#[tokio::test]
async fn test_handles_changed_room_config() -> Result<()> {
    panic!("Implement me")
}
