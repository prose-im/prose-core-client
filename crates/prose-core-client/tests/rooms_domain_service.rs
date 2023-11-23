// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use mockall::{predicate, Sequence};

use prose_core_client::domain::rooms::models::{
    RoomError, RoomInternals, RoomSessionInfo, RoomSpec,
};
use prose_core_client::domain::rooms::services::impls::RoomsDomainService;
use prose_core_client::domain::rooms::services::{
    CreateOrEnterRoomRequest, CreateRoomType, RoomsDomainService as RoomsDomainServiceTrait,
};
use prose_core_client::domain::shared::models::{RoomJid, RoomType};
use prose_core_client::dtos::{Member, PublicRoomInfo};
use prose_core_client::room;
use prose_core_client::test::{mock_data, MockRoomsDomainServiceDependencies};
use prose_xmpp::bare;
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
                    jid: room!("room@conference.prose.org"),
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
async fn test_creates_public_room_if_it_does_not_exist() -> Result<()> {
    let mut deps = MockRoomsDomainServiceDependencies::default();

    deps.id_provider = Arc::new(IncrementingIDProvider::new("hash"));

    deps.room_management_service
        .expect_load_public_rooms()
        .once()
        .return_once(|_| {
            Box::pin(async {
                Ok(vec![PublicRoomInfo {
                    jid: room!("room@conference.prose.org"),
                    name: Some("Old Channel".to_string()),
                }])
            })
        });

    deps.connected_rooms_repo
        .expect_set()
        .once()
        .with(predicate::eq(Arc::new(RoomInternals::mock_pending_room(
            room!("org.prose.public-channel.hash-1@conference.prose.org"),
            "hash-2",
        ))))
        .return_once(|_| Ok(()));

    deps.room_management_service
        .expect_create_or_join_room()
        .once()
        .return_once(|_, _, _| {
            Box::pin(async {
                Ok(RoomSessionInfo::new_room(
                    room!("org.prose.public-channel.hash-1@conference.prose.org"),
                    RoomType::PublicChannel,
                ))
            })
        });

    deps.connected_rooms_repo
        .expect_update()
        .once()
        .with(
            predicate::eq(room!(
                "org.prose.public-channel.hash-1@conference.prose.org"
            )),
            predicate::always(),
        )
        .return_once(|_, _| {
            Some(Arc::new(RoomInternals::public_channel(room!(
                "org.prose.public-channel.hash-1@conference.prose.org"
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

    let channel_jid = room!("org.prose.private-channel.hash-1@conf.prose.org");
    let full_jid = channel_jid
        .clone()
        .into_inner()
        .with_resource_str(&format!(
            "{}-hash-2",
            mock_data::account_jid().node_str().unwrap(),
        ))
        .unwrap();

    // Make sure that the method calls are in the exact order…
    let mut seq = Sequence::new();

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(room!("group@conf.prose.org")))
        .return_once(|_| {
            Some(Arc::new(
                RoomInternals::group(room!("group@conf.prose.org")).with_members(vec![
                    (
                        mock_data::account_jid().into_bare(),
                        Member {
                            name: "Jane Doe".to_string(),
                        },
                    ),
                    (
                        bare!("a@prose.org"),
                        Member {
                            name: "Member A".to_string(),
                        },
                    ),
                    (
                        bare!("b@prose.org"),
                        Member {
                            name: "Member B".to_string(),
                        },
                    ),
                ]),
            ))
        });

    deps.connected_rooms_repo
        .expect_delete()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(room!("group@conf.prose.org")))
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
                predicate::eq(full_jid),
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
            predicate::eq(room!("group@conf.prose.org")),
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
            predicate::in_iter(vec![bare!("a@prose.org"), bare!("b@prose.org")]),
        )
        .returning(|_, _| Box::pin(async { Ok(()) }));

    deps.room_management_service
        .expect_destroy_room()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(room!("group@conf.prose.org")),
            predicate::eq(Some(channel_jid.clone())),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    let service = RoomsDomainService::from(deps.into_deps());

    service
        .reconfigure_room_with_spec(
            &room!("group@conf.prose.org"),
            RoomSpec::PrivateChannel,
            "Private Channel",
        )
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_converts_private_to_public_channel() -> Result<()> {
    panic!("Implement me")
}
