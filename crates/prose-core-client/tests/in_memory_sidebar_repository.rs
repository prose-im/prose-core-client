// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::domain::shared::models::RoomJid;
use prose_core_client::domain::sidebar::models::{BookmarkType, SidebarItem};
use prose_core_client::domain::sidebar::repos::SidebarRepository;
use prose_core_client::infra::sidebar::InMemorySidebarRepository;
use prose_core_client::room;

#[tokio::test]
async fn test_put_sidebar_item() -> Result<()> {
    let repo = InMemorySidebarRepository::new();
    repo.put(&SidebarItem {
        name: "A".to_string(),
        jid: room!("a@prose.org"),
        r#type: BookmarkType::PublicChannel,
        is_favorite: false,
        error: None,
    });
    repo.put(&SidebarItem {
        name: "B".to_string(),
        jid: room!("b@prose.org"),
        r#type: BookmarkType::PublicChannel,
        is_favorite: false,
        error: None,
    });

    assert_eq!(
        repo.get_all(),
        vec![
            SidebarItem {
                name: "A".to_string(),
                jid: room!("a@prose.org"),
                r#type: BookmarkType::PublicChannel,
                is_favorite: false,
                error: None,
            },
            SidebarItem {
                name: "B".to_string(),
                jid: room!("b@prose.org"),
                r#type: BookmarkType::PublicChannel,
                is_favorite: false,
                error: None,
            }
        ]
    );

    repo.put(&SidebarItem {
        name: "C".to_string(),
        jid: room!("b@prose.org"),
        r#type: BookmarkType::PublicChannel,
        is_favorite: false,
        error: None,
    });

    assert_eq!(
        repo.get_all(),
        vec![
            SidebarItem {
                name: "A".to_string(),
                jid: room!("a@prose.org"),
                r#type: BookmarkType::PublicChannel,
                is_favorite: false,
                error: None,
            },
            SidebarItem {
                name: "C".to_string(),
                jid: room!("b@prose.org"),
                r#type: BookmarkType::PublicChannel,
                is_favorite: false,
                error: None,
            }
        ]
    );

    Ok(())
}
