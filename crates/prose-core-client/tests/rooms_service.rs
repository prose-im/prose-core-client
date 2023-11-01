// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use mockall::predicate;

use prose_core_client::domain::contacts::models::Contact;
use prose_core_client::domain::rooms::models::{Bookmark, RoomInfo, RoomInternals};
use prose_core_client::domain::rooms::repos::ConnectedRoomsRepository;
use prose_core_client::domain::rooms::services::CreateOrEnterRoomRequestType;
use prose_core_client::domain::shared::models::RoomType;
use prose_core_client::dtos::Group;
use prose_core_client::infra::rooms::InMemoryConnectedRoomsRepository;
use prose_core_client::services::RoomsService;
use prose_core_client::test::{mock_data, MockAppDependencies};
use prose_core_client::ClientEvent;
use prose_xmpp::bare;

#[tokio::test]
async fn test_creates_rooms_for_contacts_and_bookmarks() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.contacts_repo
        .expect_get_all()
        .once()
        .with(predicate::eq(mock_data::account_jid().into_bare()))
        .return_once(|_| {
            Box::pin(async {
                Ok(vec![
                    Contact {
                        jid: bare!("a@prose.org"),
                        name: None,
                        group: Group::Team,
                    },
                    Contact {
                        jid: bare!("b@prose.org"),
                        name: None,
                        group: Group::Team,
                    },
                ])
            })
        });

    deps.user_profile_repo
        .expect_get()
        .times(2)
        .returning(|_| Box::pin(async { Ok(None) }));

    deps.bookmarks_repo.expect_get_all().once().return_once(|| {
        Box::pin(async {
            Ok(vec![
                Bookmark {
                    name: "".to_string(),
                    room_jid: bare!("room1@conference.prose.org"),
                },
                Bookmark {
                    name: "".to_string(),
                    room_jid: bare!("room2@conference.prose.org"),
                },
            ])
        })
    });

    deps.rooms_domain_service
        .expect_create_or_join_room()
        .times(2)
        .returning(|req| {
            let CreateOrEnterRoomRequestType::Join { room_jid, .. } = req.r#type else {
                panic!("Expected CreateOrEnterRoomRequestType::Join")
            };

            Box::pin(async {
                Ok(Arc::new(RoomInternals {
                    info: RoomInfo {
                        jid: room_jid,
                        name: None,
                        description: None,
                        user_jid: mock_data::account_jid().into_bare(),
                        user_nickname: "".to_string(),
                        members: vec![],
                        room_type: RoomType::Group,
                    },
                    state: Default::default(),
                }))
            })
        });

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::RoomsChanged))
        .return_once(|_| ());

    let rooms_repo = Arc::new(InMemoryConnectedRoomsRepository::new());

    let mut deps = deps.into_deps();
    deps.connected_rooms_repo = rooms_repo.clone();

    let service = RoomsService::from(&deps);
    service.start_observing_rooms().await?;

    let mut created_rooms = rooms_repo.get_all();
    created_rooms.sort_by_key(|room| room.info.jid.to_string());

    assert_eq!(
        created_rooms.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
        vec![
            &RoomInternals::for_direct_message(
                &mock_data::account_jid().into_bare(),
                &Contact {
                    jid: bare!("a@prose.org"),
                    name: None,
                    group: Group::Team,
                },
                "A"
            ),
            &RoomInternals::for_direct_message(
                &mock_data::account_jid().into_bare(),
                &Contact {
                    jid: bare!("b@prose.org"),
                    name: None,
                    group: Group::Team,
                },
                "B"
            ),
            // The two missing rooms here would usually be inserted by the RoomsDomainService.
            // Not ideal but the switch from a pending room to a connected rooms needs to be atomic
            // so that we can guarantee to not lose any events targeting that room.
        ]
    );

    Ok(())
}
