// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{Arc, OnceLock};

use anyhow::Result;
use mockall::{predicate, Sequence};

use prose_core_client::domain::rooms::models::{RoomInternals, RoomSpec};
use prose_core_client::domain::rooms::services::CreateOrEnterRoomRequest;
use prose_core_client::domain::shared::models::RoomJid;
use prose_core_client::domain::sidebar::models::{Bookmark, BookmarkType, SidebarItem};
use prose_core_client::domain::sidebar::services::impls::SidebarDomainService;
use prose_core_client::domain::sidebar::services::SidebarDomainService as SidebarDomainServiceTrait;
use prose_core_client::test::MockSidebarDomainServiceDependencies;
use prose_core_client::{room, ClientEvent};
use prose_xmpp::full;

#[tokio::test]
async fn test_extends_sidebar() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(room!("group@prose.org")))
        .return_once(|_| None);

    deps.rooms_domain_service
        .expect_create_or_join_room()
        .once()
        .with(predicate::eq(CreateOrEnterRoomRequest::JoinRoom {
            room_jid: room!("group@prose.org"),
            password: None,
        }))
        .return_once(|_| {
            Box::pin(async { Ok(Arc::new(RoomInternals::group(room!("group@prose.org")))) })
        });

    deps.sidebar_repo
        .expect_put()
        .once()
        .with(predicate::eq(SidebarItem::group(
            room!("group@prose.org"),
            "Group",
        )))
        .return_once(|_| ());

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .extend_items_from_bookmarks(vec![
            Bookmark::group(room!("group@prose.org"), "Group").set_in_sidebar(true)
        ])
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_removed_item() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(room!("group@prose.org")))
        .return_once(|_| Some(SidebarItem::group(room!("group@prose.org"), "Group")));

    deps.sidebar_repo
        .expect_delete()
        .once()
        .with(predicate::eq(room!("group@prose.org")))
        .return_once(|_| ());

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .extend_items_from_bookmarks(vec![
            Bookmark::group(room!("group@prose.org"), "Group").set_in_sidebar(false)
        ])
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_does_not_add_removed_item() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(room!("group@prose.org")))
        .return_once(|_| None);

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .extend_items_from_bookmarks(vec![
            Bookmark::group(room!("group@prose.org"), "Group").set_in_sidebar(false)
        ])
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_updated_bookmark() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(room!("group@prose.org")))
        .return_once(|_| {
            Some(SidebarItem::group(room!("group@prose.org"), "Group").set_is_favorite(false))
        });

    deps.sidebar_repo
        .expect_put()
        .once()
        .with(predicate::eq(
            SidebarItem::group(room!("group@prose.org"), "Group").set_is_favorite(true),
        ))
        .return_once(|_| ());

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .extend_items_from_bookmarks(vec![Bookmark::group(room!("group@prose.org"), "Group")
            .set_in_sidebar(true)
            .set_is_favorite(true)])
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_removes_public_channel_from_sidebar() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(room!("channel@conference.prose.org")))
        .return_once(|_| {
            Some(SidebarItem::public_channel(
                room!("channel@conference.prose.org"),
                "",
            ))
        });

    deps.bookmarks_service
        .expect_delete_bookmark()
        .once()
        .with(predicate::eq(room!("channel@conference.prose.org")))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.sidebar_repo
        .expect_delete()
        .once()
        .with(predicate::eq(room!("channel@conference.prose.org")))
        .return_once(|_| ());

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(room!("channel@conference.prose.org")))
        .return_once(move |_| {
            Some(Arc::new(
                RoomInternals::public_channel(room!("channel@conference.prose.org"))
                    .with_user_nickname("jane.doe"),
            ))
        });

    deps.room_management_service
        .expect_exit_room()
        .once()
        .with(predicate::eq(full!(
            "channel@conference.prose.org/jane.doe"
        )))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .remove_items(&[&room!("channel@conference.prose.org")])
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_removes_direct_message_from_sidebar() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    deps.bookmarks_service
        .expect_delete_bookmark()
        .once()
        .with(predicate::eq(room!("contact@prose.org")))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(room!("contact@prose.org")))
        .return_once(|_| Some(SidebarItem::direct_message(room!("contact@prose.org"))));

    deps.sidebar_repo
        .expect_delete()
        .once()
        .with(predicate::eq(room!("contact@prose.org")))
        .return_once(|_| ());

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service.remove_items(&[&room!("contact@prose.org")]).await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_removed_direct_message() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(room!("contact@prose.org")))
        .return_once(|_| Some(SidebarItem::direct_message(room!("contact@prose.org"))));

    deps.sidebar_repo
        .expect_delete()
        .once()
        .with(predicate::eq(room!("contact@prose.org")))
        .return_once(|_| ());

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .handle_removed_items(&[&room!("contact@prose.org")])
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_removes_group_from_sidebar() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .with(predicate::eq(Bookmark {
            name: "Group Name".to_string(),
            jid: room!("group@conference.prose.org"),
            r#type: BookmarkType::Group,
            // The group should be removed from favorites
            is_favorite: false,
            in_sidebar: false,
        }))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(room!("group@conference.prose.org")))
        .return_once(|_| {
            Some(
                SidebarItem::group(room!("group@conference.prose.org"), "Group Name")
                    .set_is_favorite(true),
            )
        });

    deps.sidebar_repo
        .expect_delete()
        .once()
        .with(predicate::eq(room!("group@conference.prose.org")))
        .return_once(|_| ());

    // Unlike channels, groups should never be exited. This is because a Group should basically
    // behave like a Direct Message from a user perspective.

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .remove_items(&[&room!("group@conference.prose.org")])
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_removes_private_channel_from_sidebar() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .with(predicate::eq(Bookmark {
            name: "Channel Name".to_string(),
            jid: room!("channel@conference.prose.org"),
            r#type: BookmarkType::PrivateChannel,
            // The channel should be removed from favorites
            is_favorite: false,
            in_sidebar: false,
        }))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(room!("channel@conference.prose.org")))
        .return_once(|_| {
            Some(
                SidebarItem::private_channel(room!("channel@conference.prose.org"), "Channel Name")
                    .set_is_favorite(true),
            )
        });

    deps.sidebar_repo
        .expect_delete()
        .once()
        .with(predicate::eq(room!("channel@conference.prose.org")))
        .return_once(|_| ());

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(room!("channel@conference.prose.org")))
        .return_once(move |_| {
            Some(Arc::new(
                RoomInternals::private_channel(room!("channel@conference.prose.org"))
                    .with_user_nickname("jane.doe"),
            ))
        });

    deps.room_management_service
        .expect_exit_room()
        .once()
        .with(predicate::eq(full!(
            "channel@conference.prose.org/jane.doe"
        )))
        .return_once(|_| Box::pin(async { Ok(()) }));

    // Unlike public channels, private channels should never be deleted. Otherwise we cannot
    // discover it again.

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .remove_items(&[&room!("channel@conference.prose.org")])
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_insert_item_for_received_message_if_needed() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    let room =
        Arc::new(RoomInternals::group(room!("group@conference.prose.org")).with_name("Group Name"));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room!("group@conference.prose.org")))
            .return_once(|_| Some(room));
    }

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(room!("group@conference.prose.org")))
        .return_once(|_| None);

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .with(predicate::eq(Bookmark {
            name: "Group Name".to_string(),
            jid: room!("group@conference.prose.org"),
            r#type: BookmarkType::Group,
            is_favorite: false,
            in_sidebar: true,
        }))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.sidebar_repo
        .expect_put()
        .once()
        .with(predicate::eq(SidebarItem {
            name: "Group Name".to_string(),
            jid: room!("group@conference.prose.org"),
            r#type: BookmarkType::Group,
            is_favorite: false,
            error: None,
        }))
        .return_once(|_| ());

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .insert_item_for_received_message_if_needed(&room!("group@conference.prose.org"))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_renames_channel_in_sidebar() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(room!("room@conference.prose.org")))
        .return_once(|_| {
            Some(SidebarItem {
                name: "Old Name".to_string(),
                jid: room!("room@conference.prose.org"),
                r#type: BookmarkType::PublicChannel,
                is_favorite: false,
                error: None,
            })
        });

    deps.rooms_domain_service
        .expect_rename_room()
        .once()
        .with(
            predicate::eq(room!("room@conference.prose.org")),
            predicate::eq("New Name"),
        )
        .return_once(|_, _| Box::pin(async move { Ok(()) }));

    deps.sidebar_repo
        .expect_put()
        .once()
        .with(predicate::eq(SidebarItem {
            name: "New Name".to_string(),
            jid: room!("room@conference.prose.org"),
            r#type: BookmarkType::PublicChannel,
            is_favorite: false,
            error: None,
        }))
        .return_once(|_| ());

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .with(predicate::eq(Bookmark {
            name: "New Name".to_string(),
            jid: room!("room@conference.prose.org"),
            r#type: BookmarkType::PublicChannel,
            is_favorite: false,
            in_sidebar: true,
        }))
        .return_once(|_| Box::pin(async move { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .rename_item(&room!("room@conference.prose.org"), "New Name")
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_toggle_favorite() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .with(predicate::eq(Bookmark {
            name: "Channel Name".to_string(),
            jid: room!("channel@conference.prose.org"),
            r#type: BookmarkType::PublicChannel,
            is_favorite: true,
            in_sidebar: true,
        }))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(room!("channel@conference.prose.org")))
        .return_once(|_| {
            Some(SidebarItem {
                name: "Channel Name".to_string(),
                jid: room!("channel@conference.prose.org"),
                r#type: BookmarkType::PublicChannel,
                is_favorite: false,
                error: None,
            })
        });

    deps.sidebar_repo
        .expect_put()
        .once()
        .with(predicate::eq(SidebarItem {
            name: "Channel Name".to_string(),
            jid: room!("channel@conference.prose.org"),
            r#type: BookmarkType::PublicChannel,
            is_favorite: true,
            error: None,
        }))
        .return_once(|_| ());

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarDomainService::from(deps.into_deps());
    service
        .toggle_item_is_favorite(&room!("channel@conference.prose.org"))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_convert_group_to_private_channel() -> Result<()> {
    let mut deps = MockSidebarDomainServiceDependencies::default();

    // Make sure that the method calls are in the exact order…
    let mut seq = Sequence::new();

    let service = Arc::new(OnceLock::<SidebarDomainService>::new());

    {
        let service = service.clone();

        // Sequence starts in SidebarDomainService where reconfigure_item_with_spec is called.
        // The SidebarDomainService first calls into RoomsDomainService…
        deps.rooms_domain_service
            .expect_reconfigure_room_with_spec()
            .once()
            .in_sequence(&mut seq)
            .with(
                predicate::eq(room!("group@conference.prose.org")),
                predicate::eq(RoomSpec::PrivateChannel),
                predicate::eq("My Private Channel"),
            )
            .return_once(|_, _, _| {
                Box::pin(async move {
                    // RoomsDomainService then creates a new room, migrates the messages and when
                    // it finally destroys the original room, the server will send us a presence
                    // to notify us that the room was destroyed. This will be handled by
                    // the RoomsEventHandler which calls into the SidebarDomainService. So we'll
                    // simulate this here as well…
                    service
                        .get()
                        .unwrap()
                        .handle_destroyed_room(
                            &room!("group@conference.prose.org"),
                            Some(room!("private-channel@conference.prose.org")),
                        )
                        .await?;

                    Ok(Arc::new(
                        RoomInternals::private_channel(room!(
                            "private-channel@conference.prose.org"
                        ))
                        .with_name("My Private Channel"),
                    ))
                })
            });
    }

    // handle_destroyed_room
    deps.sidebar_repo
        .expect_get()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(room!("group@conference.prose.org")))
        .return_once(|_| {
            Some(SidebarItem::group(
                room!("group@conference.prose.org"),
                "My Group",
            ))
        });
    // handle_destroyed_room
    deps.connected_rooms_repo
        .expect_delete()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(room!("group@conference.prose.org")))
        .return_once(|_| ());
    // handle_destroyed_room
    deps.sidebar_repo
        .expect_delete()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(room!("group@conference.prose.org")))
        .return_once(|_| ());
    // handle_destroyed_room
    deps.bookmarks_service
        .expect_delete_bookmark()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(room!("group@conference.prose.org")))
        .return_once(|_| Box::pin(async { Ok(()) }));
    // handle_destroyed_room -> connect alternate_room
    deps.sidebar_repo
        .expect_get()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(room!("private-channel@conference.prose.org")))
        .return_once(|_| None);
    // handle_destroyed_room -> connect alternate_room
    deps.rooms_domain_service
        .expect_create_or_join_room()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(CreateOrEnterRoomRequest::JoinRoom {
            room_jid: room!("private-channel@conference.prose.org"),
            password: None,
        }))
        .return_once(|_| {
            Box::pin(async move {
                Ok(Arc::new(
                    RoomInternals::private_channel(room!("private-channel@conference.prose.org"))
                        .with_name("My Private Channel"),
                ))
            })
        });
    // handle_destroyed_room -> connect alternate_room -> insert_item_by_creating_or_joining_room -> insert_or_update_sidebar_item_and_bookmark_for_room_if_needed
    deps.sidebar_repo
        .expect_get()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(room!("private-channel@conference.prose.org")))
        .return_once(|_| None);
    // handle_destroyed_room -> connect alternate_room -> insert_item_by_creating_or_joining_room -> insert_or_update_sidebar_item_and_bookmark_for_room_if_need
    deps.sidebar_repo
        .expect_put()
        .once()
        .with(predicate::eq(SidebarItem::private_channel(
            room!("private-channel@conference.prose.org"),
            "My Private Channel",
        )))
        .return_once(|_| ());
    // handle_destroyed_room -> connect alternate_room -> insert_item_by_creating_or_joining_room -> insert_or_update_sidebar_item_and_bookmark_for_room_if_need
    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(
            Bookmark::private_channel(
                room!("private-channel@conference.prose.org"),
                "My Private Channel",
            )
            .set_in_sidebar(true),
        ))
        .return_once(|_| Box::pin(async { Ok(()) }));
    // handle_destroyed_room -> connect alternate_room -> insert_item_by_creating_or_joining_room -> insert_or_update_sidebar_item_and_bookmark_for_room_if_need
    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());
    // reconfigure_item_with_spec
    deps.sidebar_repo
        .expect_get()
        .once()
        .in_sequence(&mut seq)
        .with(predicate::eq(room!("private-channel@conference.prose.org")))
        .return_once(|_| {
            Some(SidebarItem::private_channel(
                room!("private-channel@conference.prose.org"),
                "My Private Channel",
            ))
        });

    service
        .set(SidebarDomainService::from(deps.into_deps()))
        .map_err(|_| ())
        .unwrap();

    service
        .get()
        .unwrap()
        .reconfigure_item_with_spec(
            &room!("group@conference.prose.org"),
            RoomSpec::PrivateChannel,
            "My Private Channel",
        )
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_convert_private_to_public_channel() -> Result<()> {
    panic!("Implement me!")
}

#[tokio::test]
async fn test_handle_destroyed_room() -> Result<()> {
    panic!("Implement me!")
}

#[tokio::test]
async fn test_handle_temporary_removal_from_room() -> Result<()> {
    panic!("Implement me!")
}

#[tokio::test]
async fn test_handle_permanent_removal_from_room() -> Result<()> {
    panic!("Implement me!")
}
