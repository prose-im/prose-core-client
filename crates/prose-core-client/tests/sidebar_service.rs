// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use mockall::predicate;

use prose_core_client::domain::rooms::models::RoomInternals;
use prose_core_client::domain::sidebar::models::{Bookmark, BookmarkType, SidebarItem};
use prose_core_client::services::SidebarService;
use prose_core_client::test::MockAppDependencies;
use prose_core_client::ClientEvent;
use prose_xmpp::{bare, full};

#[tokio::test]
async fn test_toggle_favorite() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .with(predicate::eq(Bookmark {
            name: "Channel Name".to_string(),
            jid: bare!("channel@conference.prose.org"),
            r#type: BookmarkType::PublicChannel,
            is_favorite: true,
            in_sidebar: true,
        }))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(bare!("channel@conference.prose.org")))
        .return_once(|_| {
            Some(SidebarItem {
                name: "Channel Name".to_string(),
                jid: bare!("channel@conference.prose.org"),
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
            jid: bare!("channel@conference.prose.org"),
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

    let service = SidebarService::from(&deps.into_deps());
    service
        .toggle_favorite(&bare!("channel@conference.prose.org"))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_removes_public_channel_from_sidebar() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.bookmarks_service
        .expect_delete_bookmark()
        .once()
        .with(predicate::eq(bare!("channel@conference.prose.org")))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(bare!("channel@conference.prose.org")))
        .return_once(|_| {
            Some(SidebarItem {
                name: "".to_string(),
                jid: bare!("channel@conference.prose.org"),
                r#type: BookmarkType::PublicChannel,
                is_favorite: false,
                error: None,
            })
        });

    deps.sidebar_repo
        .expect_delete()
        .once()
        .with(predicate::eq(bare!("channel@conference.prose.org")))
        .return_once(|_| ());

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(bare!("channel@conference.prose.org")))
        .return_once(move |_| {
            Some(Arc::new(
                RoomInternals::public_channel(&bare!("channel@conference.prose.org"))
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

    let service = SidebarService::from(&deps.into_deps());
    service
        .remove_from_sidebar(&bare!("channel@conference.prose.org"))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_removes_private_channel_from_sidebar() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .with(predicate::eq(Bookmark {
            name: "Channel Name".to_string(),
            jid: bare!("channel@conference.prose.org"),
            r#type: BookmarkType::PrivateChannel,
            // The channel should be removed from favorites
            is_favorite: false,
            in_sidebar: false,
        }))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(bare!("channel@conference.prose.org")))
        .return_once(|_| {
            Some(SidebarItem {
                name: "Channel Name".to_string(),
                jid: bare!("channel@conference.prose.org"),
                r#type: BookmarkType::PrivateChannel,
                is_favorite: true,
                error: None,
            })
        });

    deps.sidebar_repo
        .expect_delete()
        .once()
        .with(predicate::eq(bare!("channel@conference.prose.org")))
        .return_once(|_| ());

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(bare!("channel@conference.prose.org")))
        .return_once(move |_| {
            Some(Arc::new(
                RoomInternals::private_channel(&bare!("channel@conference.prose.org"))
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

    let service = SidebarService::from(&deps.into_deps());
    service
        .remove_from_sidebar(&bare!("channel@conference.prose.org"))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_removes_group_from_sidebar() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.bookmarks_service
        .expect_save_bookmark()
        .once()
        .with(predicate::eq(Bookmark {
            name: "Group Name".to_string(),
            jid: bare!("group@conference.prose.org"),
            r#type: BookmarkType::Group,
            // The group should be removed from favorites
            is_favorite: false,
            in_sidebar: false,
        }))
        .return_once(|_| Box::pin(async { Ok(()) }));

    deps.sidebar_repo
        .expect_get()
        .once()
        .with(predicate::eq(bare!("group@conference.prose.org")))
        .return_once(|_| {
            Some(SidebarItem {
                name: "Group Name".to_string(),
                jid: bare!("group@conference.prose.org"),
                r#type: BookmarkType::Group,
                is_favorite: true,
                error: None,
            })
        });

    deps.sidebar_repo
        .expect_delete()
        .once()
        .with(predicate::eq(bare!("group@conference.prose.org")))
        .return_once(|_| ());

    // Unlike channels, groups should never be exited. This is because a Group should basically
    // behave like a Direct Message from a user perspective.

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::SidebarChanged))
        .return_once(|_| ());

    let service = SidebarService::from(&deps.into_deps());
    service
        .remove_from_sidebar(&bare!("group@conference.prose.org"))
        .await?;

    Ok(())
}
