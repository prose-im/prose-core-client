// prose-core-client/prose-core-integration-tests
//
// Copyright: 2025, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::tests::{async_test, store};
use anyhow::Result;
use prose_core_client::account_id;
use prose_core_client::domain::shared::models::AccountId;
use prose_core_client::domain::workspace::models::WorkspaceInfo;
use prose_core_client::domain::workspace::repos::WorkspaceInfoRepository as _;
use prose_core_client::infra::workspace::WorkspaceInfoRepository;

#[async_test]
async fn test_loads_and_caches_workspace_infos() -> Result<()> {
    let repo = WorkspaceInfoRepository::new(store().await?);

    repo.update(
        &account_id!("a@prose.org"),
        Box::new(|info| {
            info.name = Some("Prose Server A".to_string());
        }),
    )
    .await?;

    repo.update(
        &account_id!("b@prose.org"),
        Box::new(|info| {
            info.name = Some("Prose Server B".to_string());
            info.accent_color = Some("#ff00ff".to_string());
        }),
    )
    .await?;

    assert_eq!(
        repo.get(&account_id!("a@prose.org")).await?,
        Some(WorkspaceInfo {
            name: Some("Prose Server A".to_string()),
            icon: None,
            accent_color: None,
        })
    );
    assert_eq!(
        repo.get(&account_id!("b@prose.org")).await?,
        Some(WorkspaceInfo {
            name: Some("Prose Server B".to_string()),
            icon: None,
            accent_color: Some("#ff00ff".to_string()),
        })
    );
    assert_eq!(repo.get(&account_id!("c@prose.org")).await?, None);

    repo.update(
        &account_id!("a@prose.org"),
        Box::new(|info| {
            info.accent_color = Some("#00ff00".to_string());
        }),
    )
    .await?;

    assert_eq!(
        repo.get(&account_id!("a@prose.org")).await?,
        Some(WorkspaceInfo {
            name: Some("Prose Server A".to_string()),
            icon: None,
            accent_color: Some("#00ff00".to_string()),
        })
    );

    Ok(())
}
