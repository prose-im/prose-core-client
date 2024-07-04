// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use pretty_assertions::assert_eq;

use prose_core_client::app::dtos::Availability;
use prose_core_client::domain::shared::models::{AccountId, UserId, UserResourceId};
use prose_core_client::domain::user_info::models::{Presence, UserInfo, UserStatus};
use prose_core_client::domain::user_info::repos::UserInfoRepository;
use prose_core_client::dtos::{Avatar, AvatarSource};
use prose_core_client::infra::user_info::InMemoryUserInfoRepository;
use prose_core_client::{account_id, user_id, user_resource_id};

use crate::tests::async_test;

#[async_test]
async fn test_caches_received_avatar_metadata() -> Result<()> {
    let repo = InMemoryUserInfoRepository::new();
    repo.update(
        &account_id!("user@prose.org"),
        &user_id!("a@prose.org"),
        Box::new(|info| {
            info.avatar = Some(Avatar {
                id: "fa3c5706e27f6a0093981bb315015c2bd93e094e".parse().unwrap(),
                source: AvatarSource::Pep {
                    owner: user_id!("a@prose.org").into(),
                    mime_type: "image/jpg".to_string(),
                },
            })
        }),
    )
    .await?;

    let expected_user_info = UserInfo {
        avatar: Some(Avatar {
            id: "fa3c5706e27f6a0093981bb315015c2bd93e094e".parse().unwrap(),
            source: AvatarSource::Pep {
                owner: user_id!("a@prose.org").into(),
                mime_type: "image/jpg".to_string(),
            },
        }),
        status: None,
        availability: Availability::Unavailable,
        ..Default::default()
    };

    assert_eq!(
        repo.get(&account_id!("user@prose.org"), &user_id!("a@prose.org"))
            .await?
            .as_ref(),
        Some(&expected_user_info)
    );

    Ok(())
}

#[async_test]
async fn test_persists_metadata_and_user_activity() -> Result<()> {
    let status = UserStatus {
        emoji: "🍕".to_string(),
        status: Some("Eating pizza".to_string()),
    };

    let repo = InMemoryUserInfoRepository::new();
    repo.update(
        &account_id!("user@prose.org"),
        &user_id!("a@prose.org"),
        Box::new({
            let status = status.clone();
            |info| {
                info.avatar = Some(Avatar {
                    id: "fa3c5706e27f6a0093981bb315015c2bd93e094e".parse().unwrap(),
                    source: AvatarSource::Pep {
                        owner: user_id!("a@prose.org").into(),
                        mime_type: "image/jpg".to_string(),
                    },
                });
                info.status = Some(status);
            }
        }),
    )
    .await?;

    let expected_user_info = UserInfo {
        avatar: Some(Avatar {
            id: "fa3c5706e27f6a0093981bb315015c2bd93e094e".parse().unwrap(),
            source: AvatarSource::Pep {
                owner: user_id!("a@prose.org").into(),
                mime_type: "image/jpg".to_string(),
            },
        }),
        status: Some(status),
        availability: Availability::Unavailable,
        ..Default::default()
    };

    assert_eq!(
        Some(&expected_user_info),
        repo.get(&account_id!("user@prose.org"), &user_id!("a@prose.org"))
            .await?
            .as_ref()
    );

    Ok(())
}

#[async_test]
async fn test_does_not_persist_availability() -> Result<()> {
    let repo = InMemoryUserInfoRepository::new();
    repo.set_user_presence(
        &account_id!("user@prose.org"),
        &user_resource_id!("a@prose.org/a").into(),
        &Presence {
            priority: 1,
            availability: Availability::Available,
            status: None,
            ..Default::default()
        },
    )
    .await?;

    let mut expected_user_info = UserInfo {
        avatar: None,
        status: None,
        availability: Availability::Available,
        ..Default::default()
    };

    assert_eq!(
        repo.get(&account_id!("user@prose.org"), &user_id!("a@prose.org"))
            .await?
            .as_ref(),
        Some(&expected_user_info)
    );

    let repo = InMemoryUserInfoRepository::new();

    expected_user_info.availability = Availability::Unavailable;
    assert_eq!(
        repo.get(&account_id!("user@prose.org"), &user_id!("a@prose.org"))
            .await?
            .as_ref(),
        Some(&expected_user_info)
    );

    Ok(())
}

#[async_test]
async fn test_uses_highest_presence() -> Result<()> {
    let repo = InMemoryUserInfoRepository::new();

    repo.set_user_presence(
        &account_id!("user@prose.org"),
        &user_resource_id!("a@prose.org/b").into(),
        &Presence {
            priority: 2,
            availability: Availability::Away,
            status: None,
            ..Default::default()
        },
    )
    .await?;

    repo.set_user_presence(
        &account_id!("user@prose.org"),
        &user_resource_id!("a@prose.org/a").into(),
        &Presence {
            priority: 1,
            availability: Availability::Available,
            status: None,
            ..Default::default()
        },
    )
    .await?;

    assert_eq!(
        Some(user_resource_id!("a@prose.org/b")),
        repo.resolve_user_id(&account_id!("user@prose.org"), &user_id!("a@prose.org"))
    );
    assert_eq!(
        Some(&UserInfo {
            avatar: None,
            status: None,
            availability: Availability::Away,
            ..Default::default()
        }),
        repo.get(&account_id!("user@prose.org"), &user_id!("a@prose.org"))
            .await?
            .as_ref()
    );

    Ok(())
}
