// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use mockall::predicate;

use prose_core_client::domain::rooms::models::{RoomError, RoomInternals, RoomMetadata};
use prose_core_client::domain::rooms::services::impls::RoomsDomainService;
use prose_core_client::domain::rooms::services::{
    CreateOrEnterRoomRequest, CreateRoomType, RoomsDomainService as RoomsDomainServiceTrait,
};
use prose_core_client::domain::shared::models::RoomJid;
use prose_core_client::dtos::PublicRoomInfo;
use prose_core_client::room;
use prose_core_client::test::{mock_data, MockRoomsDomainServiceDependencies};
use prose_xmpp::test::ConstantIDProvider;

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

    deps.id_provider = Arc::new(ConstantIDProvider::new("room-hash"));

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
        .with(predicate::eq(Arc::new(RoomInternals::pending(
            &room!("org.prose.public-channel.room-hash@conference.prose.org"),
            &mock_data::account_jid().into_bare(),
            mock_data::account_jid().node_str().unwrap(),
        ))))
        .return_once(|_| Ok(()));

    deps.room_management_service
        .expect_create_reserved_room()
        .once()
        .return_once(|_, _| {
            Box::pin(async {
                Ok(RoomMetadata::new_room(room!(
                    "org.prose.public-channel.room-hash@conference.prose.org"
                ))
                .with_public_channel_features())
            })
        });

    deps.connected_rooms_repo
        .expect_update()
        .once()
        .with(
            predicate::eq(room!(
                "org.prose.public-channel.room-hash@conference.prose.org"
            )),
            predicate::always(),
        )
        .return_once(|_, _| {
            Some(Arc::new(RoomInternals::public_channel(room!(
                "org.prose.public-channel.room-hash@conference.prose.org"
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
