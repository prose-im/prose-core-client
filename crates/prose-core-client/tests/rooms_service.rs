// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use mockall::predicate;
use parking_lot::Mutex;

use prose_core_client::domain::rooms::models::RoomInternals;
use prose_core_client::domain::rooms::repos::ConnectedRoomsRepository;
use prose_core_client::domain::rooms::services::CreateOrEnterRoomRequestType;
use prose_core_client::domain::sidebar::models::{Bookmark, BookmarkType, SidebarItem};
use prose_core_client::infra::rooms::InMemoryConnectedRoomsRepository;
use prose_core_client::services::RoomsService;
use prose_core_client::test::{mock_data, MockAppDependencies};
use prose_core_client::ClientEvent;
use prose_xmpp::bare;

#[tokio::test]
async fn test_connects_to_bookmarked_rooms() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.bookmarks_service
        .expect_load_bookmarks()
        .once()
        .return_once(|| {
            Box::pin(async {
                Ok(vec![
                    Bookmark {
                        name: "Jane Doe".to_string(),
                        jid: bare!("dm1@prose.org"),
                        r#type: BookmarkType::DirectMessage,
                        is_favorite: false,
                        in_sidebar: false,
                    },
                    Bookmark {
                        name: "John Doe".to_string(),
                        jid: bare!("dm2@prose.org"),
                        r#type: BookmarkType::DirectMessage,
                        is_favorite: false,
                        in_sidebar: true,
                    },
                    Bookmark {
                        name: "Group 1".to_string(),
                        jid: bare!("group1@conference.prose.org"),
                        r#type: BookmarkType::Group,
                        is_favorite: false,
                        in_sidebar: true,
                    },
                    Bookmark {
                        name: "Group 2".to_string(),
                        jid: bare!("group2@conference.prose.org"),
                        r#type: BookmarkType::Group,
                        is_favorite: false,
                        in_sidebar: false,
                    },
                    Bookmark {
                        name: "Public Channel 1".to_string(),
                        jid: bare!("pub-channel1@conference.prose.org"),
                        r#type: BookmarkType::PublicChannel,
                        is_favorite: false,
                        in_sidebar: false,
                    },
                    Bookmark {
                        name: "Public Channel 2".to_string(),
                        jid: bare!("pub-channel2@conference.prose.org"),
                        r#type: BookmarkType::PublicChannel,
                        is_favorite: true,
                        in_sidebar: true,
                    },
                    Bookmark {
                        name: "Private Channel 1".to_string(),
                        jid: bare!("priv-channel1@conference.prose.org"),
                        r#type: BookmarkType::PrivateChannel,
                        is_favorite: false,
                        in_sidebar: true,
                    },
                    Bookmark {
                        name: "Private Channel 2".to_string(),
                        jid: bare!("priv-channel2@conference.prose.org"),
                        r#type: BookmarkType::PrivateChannel,
                        is_favorite: false,
                        in_sidebar: false,
                    },
                ])
            })
        });

    let room_responses = Mutex::new(vec![
        Arc::new(RoomInternals::group(&bare!("group1@conference.prose.org"))),
        // Groups should always be connected regardless if they're in the sidebar or not.
        Arc::new(RoomInternals::group(&bare!("group2@conference.prose.org"))),
        Arc::new(RoomInternals::group(&bare!(
            "pub-channel2@conference.prose.org"
        ))),
        Arc::new(RoomInternals::group(&bare!(
            "priv-channel1@conference.prose.org"
        ))),
    ]);

    deps.rooms_domain_service
        .expect_create_or_join_room()
        .times(4)
        .returning(move |req| {
            let CreateOrEnterRoomRequestType::Join { room_jid, .. } = req.r#type else {
                panic!("Expected CreateOrEnterRoomRequestType::Join")
            };
            let response = room_responses.lock().remove(0);
            assert_eq!(room_jid, response.info.jid);
            Box::pin(async move { Ok(response) })
        });

    deps.sidebar_repo
        .expect_set_all()
        .once()
        .with(predicate::eq(vec![
            SidebarItem {
                name: "John Doe".to_string(),
                jid: bare!("dm2@prose.org"),
                r#type: BookmarkType::DirectMessage,
                is_favorite: false,
                error: None,
            },
            SidebarItem {
                name: "Group 1".to_string(),
                jid: bare!("group1@conference.prose.org"),
                r#type: BookmarkType::Group,
                is_favorite: false,
                error: None,
            },
            SidebarItem {
                name: "Public Channel 2".to_string(),
                jid: bare!("pub-channel2@conference.prose.org"),
                r#type: BookmarkType::PublicChannel,
                is_favorite: true,
                error: None,
            },
            SidebarItem {
                name: "Private Channel 1".to_string(),
                jid: bare!("priv-channel1@conference.prose.org"),
                r#type: BookmarkType::PrivateChannel,
                is_favorite: false,
                error: None,
            },
        ]))
        .return_once(|_| ());

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
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
                &bare!("dm2@prose.org"),
                "John Doe"
            ),
            // The other rooms here would usually be inserted by the RoomsDomainService.
            // Not ideal but the switch from a pending room to a connected rooms needs to be atomic
            // so that we can guarantee to not lose any events targeting that room.
        ]
    );

    Ok(())
}
