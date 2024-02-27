// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

#![feature(trait_upcasting)]

use std::sync::Arc;

use anyhow::{format_err, Result};
use mockall::{predicate, Sequence};
use parking_lot::Mutex;
use pretty_assertions::assert_eq;

use prose_core_client::domain::connection::models::{ConnectionProperties, ServerFeatures};
use prose_core_client::domain::rooms::models::{
    RegisteredMember, Room, RoomAffiliation, RoomConfig, RoomError, RoomInfo, RoomSessionInfo,
    RoomSessionMember, RoomSessionParticipant, RoomSidebarState, RoomSpec,
};
use prose_core_client::domain::rooms::services::impls::RoomsDomainService;
use prose_core_client::domain::rooms::services::{
    CreateOrEnterRoomRequest, CreateRoomBehavior, CreateRoomType, JoinRoomBehavior,
    RoomsDomainService as RoomsDomainServiceTrait,
};
use prose_core_client::domain::settings::models::AccountSettings;
use prose_core_client::domain::shared::models::{
    MucId, OccupantId, RoomId, RoomType, UserResourceId,
};
use prose_core_client::domain::sidebar::models::BookmarkType;
use prose_core_client::dtos::{
    Availability, Bookmark, Participant, ParticipantInfo, PublicRoomInfo, RoomState, UserId,
    UserInfo, UserProfile,
};
use prose_core_client::test::{mock_data, MockRoomsDomainServiceDependencies};
use prose_core_client::{muc_id, occupant_id, user_id, user_resource_id};
use prose_xmpp::bare;
use prose_xmpp::test::IncrementingIDProvider;

#[tokio::test]
async fn test_joins_room() -> Result<()> {
    let mut deps = MockRoomsDomainServiceDependencies::default();
    let mut seq = Sequence::new();

    let room = Arc::new(Mutex::new(Room::connecting(
        &muc_id!("room@conf.prose.org").into(),
        "user1#3dea7f2",
        RoomSidebarState::InSidebar,
    )));

    deps.ctx.set_connection_properties(ConnectionProperties {
        connected_jid: user_resource_id!("user1@prose.org/res"),
        server_features: Default::default(),
    });

    deps.account_settings_repo
        .expect_get()
        .once()
        .in_sequence(&mut seq)
        .return_once(|_| {
            Box::pin(async {
                Ok(AccountSettings {
                    availability: Availability::DoNotDisturb,
                    resource: None,
                })
            })
        });

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(bare!("room@conf.prose.org")))
        .return_once(|_| None);

    deps.connected_rooms_repo
        .expect_set()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(room.lock().clone()))
        .return_once(|_| Ok(()));

    deps.room_management_service
        .expect_join_room()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(occupant_id!("room@conf.prose.org/user1#3dea7f2")),
            predicate::always(),
            predicate::eq(deps.ctx.capabilities.clone()),
            predicate::eq(Availability::DoNotDisturb),
        )
        .return_once(|_, _, _, _| {
            Box::pin(async move {
                Ok(RoomSessionInfo {
                    room_id: muc_id!("room@conf.prose.org").into(),
                    config: RoomConfig {
                        room_name: Some("Room Name".to_string()),
                        room_description: None,
                        room_type: RoomType::PrivateChannel,
                    },
                    topic: Some("The Room Topic".to_string()),
                    user_nickname: "user#3dea7f2".to_string(),
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
                    participants: vec![
                        RoomSessionParticipant {
                            id: occupant_id!("room@conf.prose.org/user1#3dea7f2"),
                            is_self: true,
                            anon_id: None,
                            real_id: Some(user_id!("user1@prose.org")),
                            affiliation: RoomAffiliation::Owner,
                            availability: Availability::Available,
                        },
                        RoomSessionParticipant {
                            id: occupant_id!("room@conf.prose.org/user2#fdbda94"),
                            is_self: false,
                            anon_id: None,
                            real_id: Some(user_id!("user2@prose.org")),
                            affiliation: RoomAffiliation::Member,
                            availability: Availability::Available,
                        },
                    ],
                    room_has_been_created: false,
                })
            })
        });

    deps.user_profile_repo
        .expect_get_display_name()
        .times(3)
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
                predicate::eq(bare!("room@conf.prose.org")),
                predicate::always(),
            )
            .return_once(move |_, handler| {
                let updated_room = handler(room.lock().clone());
                *room.lock() = updated_room.clone();
                Some(updated_room)
            });
    }

    let service = Arc::new(RoomsDomainService::from(deps.into_deps()));

    service
        .create_or_join_room(
            CreateOrEnterRoomRequest::JoinRoom {
                room_id: muc_id!("room@conf.prose.org"),
                password: None,
                behavior: JoinRoomBehavior::user_initiated(),
            },
            RoomSidebarState::InSidebar,
        )
        .await?;

    assert_eq!(Some("The Room Topic".to_string()), room.lock().topic());

    let mut participants = room
        .lock()
        .participants()
        .iter()
        .map(ParticipantInfo::from)
        .collect::<Vec<_>>();
    participants.sort_by_key(|p| p.name.clone());

    assert_eq!(
        vec![
            ParticipantInfo {
                id: Some(user_id!("user1@prose.org")),
                name: "User1".to_string(),
                is_self: true,
                availability: Availability::Available,
                affiliation: RoomAffiliation::Owner
            },
            ParticipantInfo {
                id: Some(user_id!("user2@prose.org")),
                name: "User2".to_string(),
                is_self: false,
                availability: Availability::Available,
                affiliation: RoomAffiliation::Member
            },
            ParticipantInfo {
                id: Some(user_id!("user3@prose.org")),
                name: "User3".to_string(),
                is_self: false,
                availability: Availability::Unavailable,
                affiliation: RoomAffiliation::Member
            }
        ],
        participants
    );

    Ok(())
}

#[tokio::test]
async fn test_throws_conflict_error_if_room_exists() -> Result<()> {
    let mut deps = MockRoomsDomainServiceDependencies::default();

    deps.account_settings_repo
        .expect_get()
        .once()
        .return_once(|_| Box::pin(async { Ok(AccountSettings::default()) }));

    deps.room_management_service
        .expect_load_public_rooms()
        .once()
        .return_once(|_| {
            Box::pin(async {
                Ok(vec![PublicRoomInfo {
                    id: muc_id!("room@conference.prose.org").into(),
                    name: Some("new channel".to_string()),
                }])
            })
        });

    let service = RoomsDomainService::from(deps.into_deps());
    let result = service
        .create_or_join_room(
            CreateOrEnterRoomRequest::Create {
                service: mock_data::muc_service(),
                room_type: CreateRoomType::PublicChannel {
                    name: "New Channel".to_string(),
                },
                behavior: CreateRoomBehavior::FailIfGone,
            },
            RoomSidebarState::InSidebar,
        )
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

    deps.ctx.set_connection_properties(ConnectionProperties {
        connected_jid: user_resource_id!("jane.doe@prose.org/macOS"),
        server_features: Default::default(),
    });

    // jane.doe@prose.org + a@prose.org + b@prose.org + c@prose.org
    let group_id =
        muc_id!("org.prose.group.b41be06eda5bac6e7fc5ad069d6cd863c4f329eb@conference.prose.org");
    let occupant_id = group_id
        .occupant_id_with_nickname("jane.doe#3c1234b")
        .unwrap();

    let account_node = mock_data::account_jid().to_user_id().username().to_string();

    deps.account_settings_repo
        .expect_get()
        .once()
        .in_sequence(&mut seq)
        .return_once(|_| {
            Box::pin(async {
                Ok(AccountSettings {
                    availability: Availability::Away,
                    resource: None,
                })
            })
        });

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
        .expect_get()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(group_id.clone().into_inner()))
        .return_once(|_| None);

    deps.connected_rooms_repo
        .expect_set()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(Room::connecting(
            &group_id.clone().into(),
            "jane.doe#3c1234b",
            RoomSidebarState::InSidebar,
        )))
        .return_once(|_| Ok(()));
    {
        let group_jid = group_id.clone();
        deps.room_management_service
            .expect_create_or_join_room()
            .once()
            .in_sequence(&mut seq)
            .with(
                predicate::eq(occupant_id),
                predicate::eq("Jane, Tick, Track, Trick"),
                predicate::eq(RoomSpec::Group),
                predicate::eq(deps.ctx.capabilities.clone()),
                predicate::eq(Availability::Away),
            )
            .return_once(|_, _, _, _, _| {
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
            predicate::eq(group_id.clone()),
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
        let group_jid = group_id.clone();
        deps.connected_rooms_repo
            .expect_update()
            .once()
            .in_sequence(&mut seq)
            .with(
                predicate::eq(group_jid.clone().into_inner()),
                predicate::always(),
            )
            .return_once(move |_, handler| {
                let room = Room::mock_connecting_room(group_jid.clone(), "hash-1");

                let room = handler(room.clone());
                let mut members = room.participants().values().cloned().collect::<Vec<_>>();
                members.sort_by_key(|p| p.real_id.as_ref().unwrap().clone());

                assert_eq!(
                    members,
                    vec![
                        Participant {
                            real_id: Some(user_id!("a@prose.org")),
                            anon_occupant_id: None,
                            name: Some("Tick".to_string()),
                            is_self: false,
                            affiliation: RoomAffiliation::Owner,
                            availability: Default::default(),
                            compose_state: Default::default(),
                            compose_state_updated: Default::default(),
                        },
                        Participant {
                            real_id: Some(user_id!("b@prose.org")),
                            anon_occupant_id: None,
                            name: Some("Trick".to_string()),
                            is_self: false,
                            affiliation: RoomAffiliation::Owner,
                            availability: Default::default(),
                            compose_state: Default::default(),
                            compose_state_updated: Default::default(),
                        },
                        Participant {
                            real_id: Some(user_id!("c@prose.org")),
                            anon_occupant_id: None,
                            name: Some("Track".to_string()),
                            is_self: false,
                            affiliation: RoomAffiliation::Owner,
                            availability: Default::default(),
                            compose_state: Default::default(),
                            compose_state_updated: Default::default(),
                        },
                        Participant {
                            real_id: Some(user_id!("jane.doe@prose.org")),
                            anon_occupant_id: None,
                            name: Some("Jane".to_string()),
                            is_self: true,
                            affiliation: RoomAffiliation::Owner,
                            availability: Default::default(),
                            compose_state: Default::default(),
                            compose_state_updated: Default::default(),
                        }
                    ]
                );

                Some(room)
            });
    }

    deps.room_participation_service
        .expect_invite_users_to_room()
        .once()
        .with(
            predicate::eq(group_id.clone()),
            predicate::eq(vec![
                user_id!("a@prose.org"),
                user_id!("b@prose.org"),
                user_id!("c@prose.org"),
            ]),
        )
        .returning(|_, _| Box::pin(async { Ok(()) }));

    let service = RoomsDomainService::from(deps.into_deps());
    let result = service
        .create_or_join_room(
            CreateOrEnterRoomRequest::Create {
                service: mock_data::muc_service(),
                room_type: CreateRoomType::Group {
                    participants: vec![
                        user_id!("a@prose.org"),
                        user_id!("b@prose.org"),
                        user_id!("c@prose.org"),
                    ],
                },
                behavior: CreateRoomBehavior::FailIfGone,
            },
            RoomSidebarState::InSidebar,
        )
        .await;

    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_joins_direct_message() -> Result<()> {
    let mut deps = MockRoomsDomainServiceDependencies::default();
    let mut seq = Sequence::new();

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(bare!("user2@prose.org")))
        .return_once(|_| None);

    deps.user_profile_repo
        .expect_get_display_name()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(user_id!("user2@prose.org")))
        .return_once(|_| Box::pin(async { Ok(Some("Jennifer Doe".to_string())) }));

    deps.user_info_repo
        .expect_get_user_info()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(user_id!("user2@prose.org")))
        .return_once(|_| {
            Box::pin(async {
                Ok(Some(UserInfo {
                    avatar: None,
                    activity: None,
                    availability: Availability::Available,
                }))
            })
        });

    deps.connected_rooms_repo
        .expect_set_or_replace()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(Room::for_direct_message(
            &user_id!("user2@prose.org"),
            "Jennifer Doe",
            Availability::Available,
            RoomSidebarState::InSidebar,
        )))
        .return_once(|_| None);

    let service = RoomsDomainService::from(deps.into_deps());
    let room = service
        .create_or_join_room(
            CreateOrEnterRoomRequest::JoinDirectMessage {
                participant: user_id!("user2@prose.org"),
            },
            RoomSidebarState::InSidebar,
        )
        .await?;

    let mut participants = room
        .participants()
        .iter()
        .map(ParticipantInfo::from)
        .collect::<Vec<_>>();
    participants.sort_by_key(|p| p.name.clone());

    assert_eq!(
        participants,
        vec![ParticipantInfo {
            id: Some(user_id!("user2@prose.org")),
            name: "Jennifer Doe".to_string(),
            is_self: false,
            availability: Availability::Available,
            affiliation: RoomAffiliation::Owner
        },]
    );

    Ok(())
}

#[tokio::test]
async fn test_creates_public_room_if_it_does_not_exist() -> Result<()> {
    let mut deps = MockRoomsDomainServiceDependencies::default();

    deps.id_provider = Arc::new(IncrementingIDProvider::new("hash"));
    deps.ctx.set_connection_properties(ConnectionProperties {
        connected_jid: user_resource_id!("jane.doe@prose.org/macOS"),
        server_features: ServerFeatures {
            muc_service: Some(bare!("conference.prose.org")),
            http_upload_service: None,
        },
    });

    deps.account_settings_repo
        .expect_get()
        .once()
        .return_once(|_| Box::pin(async { Ok(AccountSettings::default()) }));

    deps.room_management_service
        .expect_load_public_rooms()
        .once()
        .return_once(|_| {
            Box::pin(async {
                Ok(vec![PublicRoomInfo {
                    id: muc_id!("room@conference.prose.org").into(),
                    name: Some("Old Channel".to_string()),
                }])
            })
        });

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(bare!(
            "org.prose.channel.hash-1@conference.prose.org"
        )))
        .return_once(|_| None);

    deps.connected_rooms_repo
        .expect_set()
        .once()
        .with(predicate::eq(Room::connecting(
            &muc_id!("org.prose.channel.hash-1@conference.prose.org").into(),
            "jane.doe#3c1234b",
            RoomSidebarState::InSidebar,
        )))
        .return_once(|_| Ok(()));

    deps.room_management_service
        .expect_create_or_join_room()
        .once()
        .return_once(|_, _, _, _, _| {
            Box::pin(async {
                Ok(RoomSessionInfo::new_room(
                    muc_id!("org.prose.channel.hash-1@conference.prose.org"),
                    RoomType::PublicChannel,
                ))
            })
        });

    deps.connected_rooms_repo
        .expect_update()
        .once()
        .with(
            predicate::eq(bare!("org.prose.channel.hash-1@conference.prose.org")),
            predicate::always(),
        )
        .return_once(|_, _| {
            Some(Room::mock(RoomInfo {
                room_id: muc_id!("org.prose.channel.hash-1@conference.prose.org").into(),
                user_nickname: "jane.doe#3c1234b".to_string(),
                r#type: RoomType::PublicChannel,
            }))
        });

    let service = RoomsDomainService::from(deps.into_deps());
    let result = service
        .create_or_join_room(
            CreateOrEnterRoomRequest::Create {
                service: mock_data::muc_service(),
                room_type: CreateRoomType::PublicChannel {
                    name: "New Channel".to_string(),
                },
                behavior: CreateRoomBehavior::FailIfGone,
            },
            RoomSidebarState::InSidebar,
        )
        .await;

    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_converts_group_to_private_channel() -> Result<()> {
    let mut deps = MockRoomsDomainServiceDependencies::default();
    deps.id_provider = Arc::new(IncrementingIDProvider::new("hash"));

    deps.ctx.set_connection_properties(ConnectionProperties {
        connected_jid: user_resource_id!("jane.doe@prose.org/macOS"),
        server_features: Default::default(),
    });

    let channel_id = muc_id!("org.prose.channel.hash-1@conf.prose.org");
    let occupant_id = channel_id
        .occupant_id_with_nickname("jane.doe#3c1234b")
        .unwrap();

    // Make sure that the method calls are in the exact order…
    let mut seq = Sequence::new();

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(bare!("group@conf.prose.org")))
        .return_once(|_| {
            Some(
                Room::group(muc_id!("group@conf.prose.org")).with_members(vec![
                    RegisteredMember {
                        user_id: user_id!("jane.doe@prose.org"),
                        name: Some("Jane Doe".to_string()),
                        is_self: false,
                        affiliation: RoomAffiliation::Owner,
                    },
                    RegisteredMember {
                        user_id: user_id!("a@prose.org"),
                        name: Some("Member A".to_string()),
                        is_self: false,
                        affiliation: RoomAffiliation::Owner,
                    },
                    RegisteredMember {
                        user_id: user_id!("b@prose.org"),
                        name: Some("Member B".to_string()),
                        is_self: false,
                        affiliation: RoomAffiliation::Owner,
                    },
                ]),
            )
        });

    deps.account_settings_repo
        .expect_get()
        .once()
        .return_once(|_| {
            Box::pin(async {
                Ok(AccountSettings {
                    availability: Availability::DoNotDisturb,
                    resource: None,
                })
            })
        });

    deps.connected_rooms_repo
        .expect_delete()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(bare!("group@conf.prose.org")))
        .return_once(|_| None);

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(channel_id.clone().into_inner()))
        .return_once(|_| None);

    deps.connected_rooms_repo
        .expect_set()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(Room::connecting(
            &channel_id.clone().into(),
            "jane.doe#3c1234b",
            RoomSidebarState::InSidebar,
        )))
        .return_once(|_| Ok(()));

    {
        let channel_jid = channel_id.clone();
        deps.room_management_service
            .expect_create_or_join_room()
            .once()
            .in_sequence(&mut seq)
            .with(
                predicate::eq(occupant_id),
                predicate::eq("Private Channel"),
                predicate::eq(RoomSpec::PrivateChannel),
                predicate::eq(deps.ctx.capabilities.clone()),
                predicate::eq(Availability::DoNotDisturb),
            )
            .return_once(|_, _, _, _, _| {
                Box::pin(async move {
                    Ok(RoomSessionInfo::new_room(
                        channel_jid.clone(),
                        RoomType::PrivateChannel,
                    ))
                })
            });
    }

    {
        let channel_jid = channel_id.clone();
        deps.connected_rooms_repo
            .expect_update()
            .once()
            .in_sequence(&mut seq)
            .with(
                predicate::eq(channel_jid.clone().into_inner()),
                predicate::always(),
            )
            .return_once(move |_, _| Some(Room::private_channel(channel_jid.clone())));
    }

    deps.message_migration_domain_service
        .expect_copy_all_messages_from_room()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(RoomId::from(muc_id!("group@conf.prose.org"))),
            predicate::eq(RoomId::from(channel_id.clone())),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.room_participation_service
        .expect_grant_membership()
        .times(2)
        .in_sequence(&mut seq)
        .with(
            predicate::eq(channel_id.clone()),
            predicate::in_iter(vec![user_id!("a@prose.org"), user_id!("b@prose.org")]),
        )
        .returning(|_, _| Box::pin(async { Ok(()) }));

    deps.room_management_service
        .expect_destroy_room()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(muc_id!("group@conf.prose.org")),
            predicate::eq(Some(channel_id.clone())),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    let service = RoomsDomainService::from(deps.into_deps());

    let room = service
        .reconfigure_room_with_spec(
            &muc_id!("group@conf.prose.org"),
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

    let room = Room::private_channel(muc_id!("channel@conf.prose.org")).with_members(vec![
        RegisteredMember {
            user_id: mock_data::account_jid().into_user_id(),
            name: Some("Jane Doe".to_string()),
            affiliation: RoomAffiliation::Owner,
            is_self: false,
        },
        RegisteredMember {
            user_id: user_id!("a@prose.org"),
            name: Some("Member A".to_string()),
            affiliation: RoomAffiliation::Owner,
            is_self: false,
        },
    ]);

    // Make sure that the method calls are in the exact order…
    let mut seq = Sequence::new();

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .in_sequence(&mut seq)
            .with(predicate::eq(bare!("channel@conf.prose.org")))
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
            predicate::eq(muc_id!("channel@conf.prose.org")),
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
                predicate::eq(bare!("channel@conf.prose.org")),
                predicate::always(),
            )
            .return_once(|_, handler| Some(handler(room)));
    }

    let service = RoomsDomainService::from(deps.into_deps());

    let room = service
        .reconfigure_room_with_spec(
            &muc_id!("channel@conf.prose.org"),
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
        .with(predicate::eq(bare!("channel@conf.prose.org")))
        .return_once(|_| {
            Some(
                Room::private_channel(muc_id!("channel@conf.prose.org")).with_members(vec![
                    RegisteredMember {
                        user_id: mock_data::account_jid().into_user_id(),
                        name: Some("Jane Doe".to_string()),
                        affiliation: RoomAffiliation::Owner,
                        is_self: false,
                    },
                    RegisteredMember {
                        user_id: user_id!("a@prose.org"),
                        name: Some("Member A".to_string()),
                        affiliation: RoomAffiliation::Owner,
                        is_self: false,
                    },
                ]),
            )
        });

    deps.room_management_service
        .expect_load_public_rooms()
        .once()
        .in_sequence(&mut seq)
        .return_once(|_| {
            Box::pin(async {
                Ok(vec![PublicRoomInfo {
                    id: muc_id!("room@conference.prose.org").into(),
                    name: Some("new channel".to_string()),
                }])
            })
        });

    let service = RoomsDomainService::from(deps.into_deps());

    let result = service
        .reconfigure_room_with_spec(
            &muc_id!("channel@conf.prose.org"),
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
async fn test_updates_pending_dm_message_room() -> Result<()> {
    let mut deps = MockRoomsDomainServiceDependencies::default();
    let mut seq = Sequence::new();

    deps.ctx.set_connection_properties(ConnectionProperties {
        connected_jid: user_resource_id!("user1@prose.org/res"),
        server_features: Default::default(),
    });

    let pending_room = Room::pending(
        &Bookmark {
            name: "".to_string(),
            jid: user_id!("user2@prose.org").into(),
            r#type: BookmarkType::DirectMessage,
            sidebar_state: RoomSidebarState::InSidebar,
        },
        "user1#3dea7f2",
    );

    {
        let pending_room = pending_room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .in_sequence(&mut seq)
            .with(predicate::eq(bare!("user2@prose.org")))
            .return_once(|_| Some(pending_room));
    }

    deps.user_profile_repo
        .expect_get_display_name()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(user_id!("user2@prose.org")))
        .return_once(|_| Box::pin(async { Ok(Some("Jennifer Doe".to_string())) }));

    deps.user_info_repo
        .expect_get_user_info()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(user_id!("user2@prose.org")))
        .return_once(|_| {
            Box::pin(async {
                Ok(Some(UserInfo {
                    avatar: None,
                    activity: None,
                    availability: Availability::Available,
                }))
            })
        });

    deps.connected_rooms_repo
        .expect_set_or_replace()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(Room::for_direct_message(
            &user_id!("user2@prose.org"),
            "Jennifer Doe",
            Availability::Available,
            RoomSidebarState::InSidebar,
        )))
        .return_once(|_| Some(pending_room));

    let service = RoomsDomainService::from(deps.into_deps());
    let room = service
        .create_or_join_room(
            CreateOrEnterRoomRequest::JoinDirectMessage {
                participant: user_id!("user2@prose.org"),
            },
            RoomSidebarState::InSidebar,
        )
        .await?;

    let participants = room
        .participants()
        .iter()
        .map(ParticipantInfo::from)
        .collect::<Vec<_>>();

    assert_eq!(room.state(), RoomState::Connected);
    assert_eq!(
        participants,
        vec![ParticipantInfo {
            id: Some(user_id!("user2@prose.org")),
            name: "Jennifer Doe".to_string(),
            is_self: false,
            availability: Availability::Available,
            affiliation: RoomAffiliation::Owner
        },]
    );

    Ok(())
}

#[tokio::test]
async fn test_updates_pending_public_channel() -> Result<()> {
    let mut deps = MockRoomsDomainServiceDependencies::default();
    let mut seq = Sequence::new();

    deps.ctx.set_connection_properties(ConnectionProperties {
        connected_jid: user_resource_id!("user1@prose.org/res"),
        server_features: Default::default(),
    });

    let pending_room = Arc::new(Mutex::new(Room::pending(
        &Bookmark {
            name: "Pending Channel Name".to_string(),
            jid: muc_id!("room@conf.prose.org").into(),
            r#type: BookmarkType::PublicChannel,
            sidebar_state: RoomSidebarState::InSidebar,
        },
        "user1#3dea7f2",
    )));

    {
        let pending_room = pending_room.lock().clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(bare!("room@conf.prose.org")))
            .in_sequence(&mut seq)
            .return_once(|_| Some(pending_room));
    }

    deps.account_settings_repo
        .expect_get()
        .once()
        .return_once(|_| Box::pin(async { Ok(AccountSettings::default()) }));

    deps.room_management_service
        .expect_join_room()
        .once()
        .with(
            predicate::eq(occupant_id!("room@conf.prose.org/user1#3dea7f2")),
            predicate::always(),
            predicate::always(),
            predicate::always(),
        )
        .in_sequence(&mut seq)
        .return_once(|_, _, _, _| {
            Box::pin(async {
                Ok(RoomSessionInfo {
                    room_id: muc_id!("room@conf.prose.org"),
                    config: RoomConfig {
                        room_name: Some("Updated Channel Name".to_string()),
                        room_description: None,
                        room_type: RoomType::PublicChannel,
                    },
                    topic: None,
                    user_nickname: "user#3dea7f2".to_string(),
                    members: vec![
                        RoomSessionMember {
                            id: user_id!("user1@prose.org"),
                            affiliation: RoomAffiliation::Owner,
                        },
                        RoomSessionMember {
                            id: user_id!("user2@prose.org"),
                            affiliation: RoomAffiliation::Member,
                        },
                    ],
                    participants: vec![],
                    room_has_been_created: false,
                })
            })
        });

    deps.user_profile_repo
        .expect_get_display_name()
        .times(2)
        .in_sequence(&mut seq)
        .with(predicate::in_iter([
            user_id!("user1@prose.org"),
            user_id!("user2@prose.org"),
        ]))
        .returning(|user_id| {
            let username = user_id.formatted_username();
            Box::pin(async move { Ok(Some(username)) })
        });

    {
        let room = pending_room.clone();
        deps.connected_rooms_repo
            .expect_update()
            .once()
            .in_sequence(&mut seq)
            .with(
                predicate::eq(bare!("room@conf.prose.org")),
                predicate::always(),
            )
            .return_once(move |_, handler| {
                let updated_room = handler(room.lock().clone());
                *room.lock() = updated_room.clone();
                Some(updated_room)
            });
    }

    let service = RoomsDomainService::from(deps.into_deps());
    service
        .create_or_join_room(
            CreateOrEnterRoomRequest::JoinRoom {
                room_id: muc_id!("room@conf.prose.org"),
                password: None,
                behavior: JoinRoomBehavior::user_initiated(),
            },
            RoomSidebarState::InSidebar,
        )
        .await?;

    let room = pending_room.lock();
    assert_eq!(room.name(), Some("Updated Channel Name".to_string()));
    assert_eq!(room.participants().len(), 2);
    assert_eq!(room.state(), RoomState::Connected);

    Ok(())
}

#[tokio::test]
async fn test_join_retains_room_on_failure() -> Result<()> {
    let mut deps = MockRoomsDomainServiceDependencies::default();
    let mut seq = Sequence::new();

    let retained_room = Arc::new(Mutex::new(Option::<Room>::None));

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(bare!("room@conf.prose.org")))
        .in_sequence(&mut seq)
        .return_once(|_| None);

    deps.account_settings_repo
        .expect_get()
        .once()
        .return_once(|_| Box::pin(async { Ok(AccountSettings::default()) }));

    {
        let retained_room = retained_room.clone();
        deps.connected_rooms_repo
            .expect_set()
            .once()
            .in_sequence(&mut seq)
            .return_once(move |room| {
                retained_room.lock().replace(room);
                Ok(())
            });
    }

    deps.room_management_service
        .expect_join_room()
        .once()
        .in_sequence(&mut seq)
        .return_once(|_, _, _, _| {
            Box::pin(async { Err(RoomError::Anyhow(format_err!("failure-error-message"))) })
        });

    let service = RoomsDomainService::from(deps.into_deps());
    let result = service
        .create_or_join_room(
            CreateOrEnterRoomRequest::JoinRoom {
                room_id: muc_id!("room@conf.prose.org"),
                password: None,
                behavior: JoinRoomBehavior::system_initiated(),
            },
            RoomSidebarState::InSidebar,
        )
        .await;

    assert!(result.is_err());
    assert_eq!(
        retained_room.lock().take().unwrap().state(),
        RoomState::Disconnected {
            error: Some("failure-error-message".into()),
            can_retry: true
        }
    );

    Ok(())
}

#[tokio::test]
async fn test_join_removes_room_on_failure() -> Result<()> {
    let mut deps = MockRoomsDomainServiceDependencies::default();
    let mut seq = Sequence::new();

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(bare!("room@conf.prose.org")))
        .in_sequence(&mut seq)
        .return_once(|_| None);

    deps.account_settings_repo
        .expect_get()
        .once()
        .return_once(|_| Box::pin(async { Ok(AccountSettings::default()) }));

    deps.connected_rooms_repo
        .expect_set()
        .once()
        .in_sequence(&mut seq)
        .return_once(move |_| Ok(()));

    deps.room_management_service
        .expect_join_room()
        .once()
        .in_sequence(&mut seq)
        .return_once(|_, _, _, _| {
            Box::pin(async { Err(RoomError::Anyhow(format_err!("failure-error-message"))) })
        });

    deps.connected_rooms_repo
        .expect_delete()
        .once()
        .with(predicate::eq(bare!("room@conf.prose.org")))
        .in_sequence(&mut seq)
        .return_once(|_| None);

    let service = RoomsDomainService::from(deps.into_deps());
    let result = service
        .create_or_join_room(
            CreateOrEnterRoomRequest::JoinRoom {
                room_id: muc_id!("room@conf.prose.org"),
                password: None,
                behavior: JoinRoomBehavior::user_initiated(),
            },
            RoomSidebarState::InSidebar,
        )
        .await;

    assert!(result.is_err());

    Ok(())
}
