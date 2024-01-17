// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use mockall::{predicate, Sequence};

use prose_core_client::domain::connection::models::ConnectionProperties;
use prose_core_client::domain::rooms::models::{RoomInternals, RoomSidebarState, RoomSpec};
use prose_core_client::domain::rooms::services::{CreateOrEnterRoomRequest, JoinRoomBehavior};
use prose_core_client::domain::shared::models::{
    OccupantId, RoomId, UserEndpointId, UserId, UserResourceId,
};
use prose_core_client::domain::sidebar::models::{Bookmark, BookmarkType};
use prose_core_client::domain::sidebar::services::impls::SidebarDomainService;
use prose_core_client::domain::sidebar::services::SidebarDomainService as SidebarDomainServiceTrait;
use prose_core_client::dtos::{Availability, RoomState};
use prose_core_client::test::{DisconnectedState, MockSidebarDomainServiceDependencies};
use prose_core_client::{occupant_id, room_id, user_id, user_resource_id, ClientEvent};

#[tokio::test]
async fn test_extend_items_insert_items() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    let room1 = Arc::new(
        RoomInternals::public_channel(room_id!("channel1@prose.org"))
            .with_user_nickname("user-nickname")
            .with_name("Channel 1")
            .with_sidebar_state(RoomSidebarState::InSidebar),
    );
    let room2 = Arc::new(
        RoomInternals::public_channel(room_id!("channel2@prose.org"))
            .with_user_nickname("user-nickname")
            .with_name("Channel 2")
            .with_sidebar_state(RoomSidebarState::InSidebar),
    );

    deps.ctx.set_connection_properties(ConnectionProperties {
        connected_jid: user_resource_id!("user1@prose.org/res"),
        server_features: Default::default(),
    });

    {
        let rooms = vec![room2.clone()];
        deps.connected_rooms_repo
            .expect_get_all()
            .once()
            .return_once(|| rooms);
    }

    deps.connected_rooms_repo
        .expect_set()
        .once()
        .with(predicate::eq(Arc::new(RoomInternals::pending(
            &Bookmark::try_from(room1.as_ref()).unwrap(),
            "user1#3dea7f2",
        ))))
        .return_once(|_| Ok(()));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    {
        let room1 = room1.clone();
        deps.rooms_domain_service
            .expect_create_or_join_room()
            .once()
            .with(
                predicate::eq(CreateOrEnterRoomRequest::JoinRoom {
                    room_id: room_id!("channel1@prose.org"),
                    password: None,
                    behavior: JoinRoomBehavior::system_initiated(),
                }),
                predicate::eq(RoomSidebarState::InSidebar),
            )
            .return_once(|_, _| Box::pin(async { Ok(room1) }));
    }

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .extend_items_from_bookmarks(vec![
            Bookmark::try_from(room1.as_ref()).unwrap(),
            Bookmark::try_from(room2.as_ref()).unwrap(),
        ])
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_extend_items_removed_item() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    let room1 = Arc::new(
        RoomInternals::public_channel(room_id!("channel1@prose.org"))
            .with_user_nickname("user-nickname")
            .with_name("Channel 1")
            .with_sidebar_state(RoomSidebarState::InSidebar),
    );
    let room2 = Arc::new(
        RoomInternals::public_channel(room_id!("channel2@prose.org"))
            .with_user_nickname("user-nickname")
            .with_name("Channel 2")
            .with_sidebar_state(RoomSidebarState::InSidebar),
    );

    {
        let rooms = vec![room1.clone(), room2.clone()];
        deps.connected_rooms_repo
            .expect_get_all()
            .once()
            .return_once(|| rooms);
    }

    deps.room_management_service
        .expect_exit_room()
        .once()
        .with(predicate::eq(occupant_id!(
            "channel1@prose.org/user-nickname"
        )))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.connected_rooms_repo
        .expect_delete()
        .once()
        .with(predicate::eq(room_id!("channel1@prose.org")))
        .return_once(|_| Some(room1));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .extend_items_from_bookmarks(vec![Bookmark::try_from(room2.as_ref()).unwrap()])
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_updated_bookmark() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();
    let mut seq = Sequence::new();

    let room = Arc::new(
        RoomInternals::group(room_id!("group@prose.org"))
            .with_name("Group")
            .with_sidebar_state(RoomSidebarState::InSidebar),
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get_all()
            .once()
            .in_sequence(&mut seq)
            .return_once(|| vec![room]);
    }

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .extend_items_from_bookmarks(vec![Bookmark::group(room_id!("group@prose.org"), "Group")
            .set_sidebar_state(RoomSidebarState::Favorite)])
        .await?;

    assert_eq!(room.sidebar_state(), RoomSidebarState::Favorite);

    Ok(())
}

#[tokio::test]
async fn test_removes_public_channel_from_sidebar() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(room_id!("channel@conference.prose.org")))
        .return_once(move |_| {
            Some(Arc::new(
                RoomInternals::public_channel(room_id!("channel@conference.prose.org"))
                    .with_name("Channel Name"),
            ))
        });

    deps.connected_rooms_repo
        .expect_delete()
        .once()
        .with(predicate::eq(room_id!("channel@conference.prose.org")))
        .return_once(|_| None);

    deps.room_management_service
        .expect_exit_room()
        .once()
        .with(predicate::eq(occupant_id!(
            "channel@conference.prose.org/jane.doe"
        )))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.bookmarks_service
        .expect_delete_bookmark()
        .once()
        .with(predicate::eq(room_id!("channel@conference.prose.org")))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .remove_items(&[&room_id!("channel@conference.prose.org")])
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_removes_direct_message_from_sidebar() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    let room = Arc::new(
        RoomInternals::direct_message(user_id!("contact@prose.org"), Availability::Available)
            .with_name("Jane Doe")
            .with_sidebar_state(RoomSidebarState::InSidebar),
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room_id!("contact@prose.org")))
            .return_once(move |_| Some(room));
    }

    deps.bookmarks_service
        .expect_delete_bookmark()
        .once()
        .with(predicate::eq(room_id!("contact@prose.org")))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.connected_rooms_repo
        .expect_delete()
        .once()
        .with(predicate::eq(room_id!("contact@prose.org")))
        .return_once(|_| Some(room));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .remove_items(&[&room_id!("contact@prose.org")])
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_removed_direct_message() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    let room = Arc::new(RoomInternals::direct_message(
        user_id!("contact@prose.org"),
        Availability::Unavailable,
    ));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room_id!("contact@prose.org")))
            .return_once(|_| Some(room));
    }

    deps.connected_rooms_repo
        .expect_delete()
        .once()
        .with(predicate::eq(room_id!("contact@prose.org")))
        .return_once(|_| Some(room));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .handle_removed_items(&[room_id!("contact@prose.org")])
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_removes_group_from_sidebar() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    let room = Arc::new(
        RoomInternals::group(room_id!("group@conference.prose.org"))
            .with_name("Group Name")
            .with_sidebar_state(RoomSidebarState::InSidebar),
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room_id!("group@conference.prose.org")))
            .return_once(|_| Some(room));
    }

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .with(predicate::eq(Bookmark {
            name: "Group Name".to_string(),
            jid: room_id!("group@conference.prose.org"),
            r#type: BookmarkType::Group,
            sidebar_state: RoomSidebarState::NotInSidebar,
        }))
        .return_once(|_| Box::pin(async { Ok(()) }));

    // Unlike channels, groups should never be exited. This is because a Group should basically
    // behave like a Direct Message from a user perspective.

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .remove_items(&[&room_id!("group@conference.prose.org")])
        .await?;

    assert_eq!(room.sidebar_state(), RoomSidebarState::NotInSidebar);

    Ok(())
}

#[tokio::test]
async fn test_removes_private_channel_from_sidebar() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    let room = Arc::new(
        RoomInternals::private_channel(room_id!("channel@conference.prose.org"))
            .with_name("Channel Name")
            .with_sidebar_state(RoomSidebarState::InSidebar),
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room_id!("channel@conference.prose.org")))
            .return_once(|_| Some(room));
    }

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .with(predicate::eq(Bookmark {
            name: "Channel Name".to_string(),
            jid: room_id!("channel@conference.prose.org"),
            r#type: BookmarkType::PrivateChannel,
            sidebar_state: RoomSidebarState::NotInSidebar,
        }))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.room_management_service
        .expect_exit_room()
        .once()
        .with(predicate::eq(occupant_id!(
            "channel@conference.prose.org/jane.doe"
        )))
        .return_once(|_| Box::pin(async { Ok(()) }));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_delete()
            .once()
            .with(predicate::eq(room_id!("channel@conference.prose.org")))
            .return_once(|_| Some(room));
    }

    // Unlike public channels, private channels should never be deleted. Otherwise we cannot
    // discover it again.

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .remove_items(&[&room_id!("channel@conference.prose.org")])
        .await?;

    assert_eq!(room.sidebar_state(), RoomSidebarState::NotInSidebar);

    Ok(())
}

#[tokio::test]
async fn test_insert_item_for_received_group_message_if_needed() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    let room = Arc::new(
        RoomInternals::group(room_id!("group@conference.prose.org"))
            .with_name("Group Name")
            .with_sidebar_state(RoomSidebarState::NotInSidebar),
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room_id!("group@conference.prose.org")))
            .return_once(|_| Some(room));
    }

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .with(predicate::eq(Bookmark {
            name: "Group Name".to_string(),
            jid: room_id!("group@conference.prose.org"),
            r#type: BookmarkType::Group,
            sidebar_state: RoomSidebarState::InSidebar,
        }))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .insert_item_for_received_message_if_needed(&UserEndpointId::Occupant(occupant_id!(
            "group@conference.prose.org/user"
        )))
        .await?;

    assert_eq!(room.sidebar_state(), RoomSidebarState::InSidebar);

    Ok(())
}

#[tokio::test]
async fn test_insert_item_for_received_direct_message_if_needed() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(room_id!("contact@prose.org")))
        .return_once(|_| None);

    deps.rooms_domain_service
        .expect_create_or_join_room()
        .once()
        .with(
            predicate::eq(CreateOrEnterRoomRequest::JoinDirectMessage {
                participant: user_id!("contact@prose.org"),
            }),
            predicate::eq(RoomSidebarState::NotInSidebar),
        )
        .return_once(|_, _| {
            Box::pin(async {
                Ok(Arc::new(
                    RoomInternals::direct_message(
                        user_id!("contact@prose.org"),
                        Availability::Available,
                    )
                    .with_name("Jane Doe")
                    .with_sidebar_state(RoomSidebarState::NotInSidebar),
                ))
            })
        });

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .with(predicate::eq(Bookmark {
            name: "Jane Doe".to_string(),
            jid: room_id!("contact@prose.org"),
            r#type: BookmarkType::DirectMessage,
            sidebar_state: RoomSidebarState::InSidebar,
        }))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .insert_item_for_received_message_if_needed(&UserEndpointId::UserResource(
            user_resource_id!("contact@prose.org/res"),
        ))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_renames_channel_in_sidebar() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    let room = Arc::new(
        RoomInternals::public_channel(room_id!("room@conference.prose.org"))
            .with_name("Channel Name")
            .with_sidebar_state(RoomSidebarState::Favorite),
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room_id!("room@conference.prose.org")))
            .return_once(|_| Some(room));
    }

    deps.rooms_domain_service
        .expect_rename_room()
        .once()
        .with(
            predicate::eq(room_id!("room@conference.prose.org")),
            predicate::eq("New Name"),
        )
        .return_once(|_, _| Box::pin(async move { Ok(()) }));

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .with(predicate::eq(Bookmark {
            name: "New Name".to_string(),
            jid: room_id!("room@conference.prose.org"),
            r#type: BookmarkType::PublicChannel,
            sidebar_state: RoomSidebarState::Favorite,
        }))
        .return_once(|_| Box::pin(async move { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .rename_item(&room_id!("room@conference.prose.org"), "New Name")
        .await?;

    assert_eq!(room.name(), Some("New Name".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_toggle_favorite() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    let room = Arc::new(
        RoomInternals::public_channel(room_id!("channel@conference.prose.org"))
            .with_name("Channel Name")
            .with_sidebar_state(RoomSidebarState::InSidebar),
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room_id!("channel@conference.prose.org")))
            .return_once(|_| Some(room));
    }

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .with(predicate::eq(Bookmark {
            name: "Channel Name".to_string(),
            jid: room_id!("channel@conference.prose.org"),
            r#type: BookmarkType::PublicChannel,
            sidebar_state: RoomSidebarState::Favorite,
        }))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .toggle_item_is_favorite(&room_id!("channel@conference.prose.org"))
        .await?;

    assert_eq!(room.sidebar_state(), RoomSidebarState::Favorite);

    Ok(())
}

#[tokio::test]
async fn test_convert_group_to_private_channel() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    // Make sure that the method calls are in the exact order…
    let mut seq = Sequence::new();

    // Sequence starts in SidebarDomainService where reconfigure_item_with_spec is called.
    // The SidebarDomainService first calls into RoomsDomainService…
    deps.rooms_domain_service
        .expect_reconfigure_room_with_spec()
        .once()
        .in_sequence(&mut seq)
        .with(
            predicate::eq(room_id!("group@conference.prose.org")),
            predicate::eq(RoomSpec::PrivateChannel),
            predicate::eq("My Private Channel"),
        )
        .return_once(|_, _, _| {
            Box::pin(async move {
                // RoomsDomainService then creates a new room, migrates the messages and when
                // it finally destroys the original room, the server will send us a presence
                // to notify us that the room was destroyed. This will be handled by
                // the RoomsEventHandler but the room will be removed from the
                // ConnectedRoomsRepository already, so this will not be forwarded to
                // the SidebarDomainService.
                Ok(Arc::new(
                    RoomInternals::private_channel(room_id!(
                        "private-channel@conference.prose.org"
                    ))
                    .with_name("My Private Channel")
                    .with_sidebar_state(RoomSidebarState::InSidebar),
                ))
            })
        });

    deps.bookmarks_service
        .expect_delete_bookmark()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(room_id!("group@conference.prose.org")))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(
            Bookmark::private_channel(
                room_id!("private-channel@conference.prose.org"),
                "My Private Channel",
            )
            .set_sidebar_state(RoomSidebarState::InSidebar),
        ))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .reconfigure_item_with_spec(
            &room_id!("group@conference.prose.org"),
            RoomSpec::PrivateChannel,
            "My Private Channel",
        )
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_destroyed_room_with_alternate_room() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    // Make sure that the method calls are in the exact order…
    let mut seq = Sequence::new();

    deps.ctx.set_connection_properties(ConnectionProperties {
        connected_jid: user_resource_id!("user1@prose.org/res"),
        server_features: Default::default(),
    });

    deps.connected_rooms_repo
        .expect_delete()
        .once()
        .with(predicate::eq(room_id!("group@muc.prose.org")))
        .in_sequence(&mut seq)
        .return_once(|_| {
            Some(Arc::new(
                RoomInternals::group(room_id!("group@muc.prose.org"))
                    .with_name("Destroyed Group")
                    .with_sidebar_state(RoomSidebarState::Favorite),
            ))
        });

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(room_id!("channel@muc.prose.org")))
        .in_sequence(&mut seq)
        .return_once(|_| None);

    deps.connected_rooms_repo
        .expect_set()
        .once()
        .with(predicate::eq(Arc::new(RoomInternals::pending(
            &Bookmark {
                name: "Destroyed Group".to_string(),
                jid: room_id!("channel@muc.prose.org"),
                r#type: BookmarkType::Group,
                sidebar_state: RoomSidebarState::Favorite,
            },
            "user1#3dea7f2",
        ))))
        .return_once(|_| Ok(()));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    deps.bookmarks_service
        .expect_delete_bookmark()
        .once()
        .with(predicate::eq(room_id!("group@muc.prose.org")))
        .in_sequence(&mut seq)
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.rooms_domain_service
        .expect_create_or_join_room()
        .once()
        .with(
            predicate::eq(CreateOrEnterRoomRequest::JoinRoom {
                room_id: room_id!("channel@muc.prose.org"),
                password: None,
                behavior: JoinRoomBehavior::system_initiated(),
            }),
            predicate::eq(RoomSidebarState::Favorite),
        )
        .in_sequence(&mut seq)
        .return_once(|_, _| {
            Box::pin(async {
                Ok(Arc::new(
                    RoomInternals::private_channel(room_id!("channel@muc.prose.org"))
                        .with_name("The Channel")
                        .with_sidebar_state(RoomSidebarState::Favorite),
                ))
            })
        });

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(Bookmark {
            name: "The Channel".to_string(),
            jid: room_id!("channel@muc.prose.org"),
            r#type: BookmarkType::PrivateChannel,
            sidebar_state: RoomSidebarState::Favorite,
        }))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .handle_destroyed_room(
            &room_id!("group@muc.prose.org"),
            Some(room_id!("channel@muc.prose.org")),
        )
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_destroyed_room_without_alternate_room() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();
    let mut seq = Sequence::new();

    let room = Arc::new(
        RoomInternals::private_channel(room_id!("room@conf.prose.org"))
            .with_state(RoomState::Connected),
    );
    assert_eq!(
        room.is_disconnected(),
        DisconnectedState {
            is_disconnected: false,
            can_retry: false
        }
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .in_sequence(&mut seq)
            .with(predicate::eq(room_id!("room@conf.prose.org")))
            .return_once(|_| Some(room));
    }

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .handle_destroyed_room(&room_id!("room@conf.prose.org"), None)
        .await?;

    assert_eq!(
        room.is_disconnected(),
        DisconnectedState {
            is_disconnected: true,
            can_retry: false
        }
    );

    Ok(())
}

#[tokio::test]
async fn test_handles_temporary_removal_from_room() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();
    let mut seq = Sequence::new();

    let room = Arc::new(
        RoomInternals::private_channel(room_id!("room@conf.prose.org"))
            .with_state(RoomState::Connected),
    );
    assert_eq!(
        room.is_disconnected(),
        DisconnectedState {
            is_disconnected: false,
            can_retry: false
        }
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .in_sequence(&mut seq)
            .with(predicate::eq(room_id!("room@conf.prose.org")))
            .return_once(|_| Some(room));
    }

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .handle_removal_from_room(&room_id!("room@conf.prose.org"), false)
        .await?;

    assert_eq!(
        room.is_disconnected(),
        DisconnectedState {
            is_disconnected: true,
            can_retry: true
        }
    );

    Ok(())
}

#[tokio::test]
async fn test_handles_permanent_removal_from_room() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();
    let mut seq = Sequence::new();

    let room = Arc::new(
        RoomInternals::private_channel(room_id!("room@conf.prose.org"))
            .with_state(RoomState::Connected),
    );
    assert_eq!(
        room.is_disconnected(),
        DisconnectedState {
            is_disconnected: false,
            can_retry: false
        }
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .in_sequence(&mut seq)
            .with(predicate::eq(room_id!("room@conf.prose.org")))
            .return_once(|_| Some(room));
    }

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .handle_removal_from_room(&room_id!("room@conf.prose.org"), true)
        .await?;

    assert_eq!(
        room.is_disconnected(),
        DisconnectedState {
            is_disconnected: true,
            can_retry: false
        }
    );

    Ok(())
}

#[tokio::test]
async fn test_handles_changed_room_config() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    // Make sure that the method calls are in the exact order…
    let mut seq = Sequence::new();

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(room_id!("room@conf.prose.org")))
        .return_once(move |_| {
            Some(Arc::new(
                RoomInternals::private_channel(room_id!("room@conf.prose.org"))
                    .with_name("Old Room Name")
                    .with_sidebar_state(RoomSidebarState::InSidebar),
            ))
        });

    deps.rooms_domain_service
        .expect_reevaluate_room_spec()
        .with(predicate::eq(room_id!("room@conf.prose.org")))
        .once()
        .in_sequence(&mut seq)
        .return_once(|_| {
            Box::pin(async {
                Ok(Arc::new(
                    RoomInternals::private_channel(room_id!("room@conf.prose.org"))
                        .with_name("New Room Name")
                        .with_sidebar_state(RoomSidebarState::InSidebar),
                ))
            })
        });

    deps.bookmarks_service
        .expect_save_bookmark()
        .with(predicate::eq(
            Bookmark::private_channel(room_id!("room@conf.prose.org"), "New Room Name")
                .set_sidebar_state(RoomSidebarState::InSidebar),
        ))
        .once()
        .in_sequence(&mut seq)
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .handle_changed_room_config(&room_id!("room@conf.prose.org"))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_ignores_changed_config_for_connecting_room() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(room_id!("room@conf.prose.org")))
        .return_once(move |_| {
            Some(Arc::new(RoomInternals::connecting(
                &room_id!("room@conf.prose.org"),
                "nick",
                RoomSidebarState::InSidebar,
            )))
        });

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .handle_changed_room_config(&room_id!("room@conf.prose.org"))
        .await?;

    Ok(())
}
