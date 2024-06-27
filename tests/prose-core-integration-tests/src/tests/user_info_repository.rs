// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;

use prose_core_client::app::dtos::Availability;
use prose_core_client::domain::shared::models::{AccountId, UserId, UserResourceId};
use prose_core_client::domain::user_info::models::{
    AvatarMetadata, Presence, UserInfo, UserStatus,
};
use prose_core_client::domain::user_info::repos::UserInfoRepository;
use prose_core_client::domain::user_info::services::mocks::MockUserInfoService;
use prose_core_client::dtos::{Avatar, AvatarSource};
use prose_core_client::infra::user_info::CachingUserInfoRepository;
use prose_core_client::{account_id, user_id, user_resource_id};

use crate::tests::{async_test, store};

#[async_test]
async fn test_caches_loaded_avatar_metadata() -> Result<()> {
    let metadata = AvatarMetadata {
        bytes: 1001,
        mime_type: "image/jpg".to_string(),
        checksum: "fa3c5706e27f6a0093981bb315015c2bd93e094e".parse().unwrap(),
        width: None,
        height: None,
        url: None,
    };

    let service = {
        let metadata = metadata.clone();
        let mut service = MockUserInfoService::new();
        service
            .expect_load_latest_avatar_metadata()
            .times(1)
            .return_once(|_| Box::pin(async { Ok(Some(metadata)) }));
        service
    };

    let repo = CachingUserInfoRepository::new(store().await?, Arc::new(service));

    let expected_user_info = UserInfo {
        avatar: Some(Avatar {
            id: "fa3c5706e27f6a0093981bb315015c2bd93e094e".parse().unwrap(),
            source: AvatarSource::Pep {
                mime_type: "image/jpg".to_string(),
            },
            owner: user_id!("a@prose.org").into(),
        }),
        activity: None,
        availability: Availability::Unavailable,
    };

    assert_eq!(
        repo.get_user_info(&account_id!("user@prose.org"), &user_id!("a@prose.org"))
            .await?
            .as_ref(),
        Some(&expected_user_info)
    );
    assert_eq!(
        repo.get_user_info(&account_id!("user@prose.org"), &user_id!("a@prose.org"))
            .await?
            .as_ref(),
        Some(&expected_user_info)
    );

    Ok(())
}

#[async_test]
async fn test_caches_received_avatar_metadata() -> Result<()> {
    let metadata = AvatarMetadata {
        bytes: 1001,
        mime_type: "image/jpg".to_string(),
        checksum: "fa3c5706e27f6a0093981bb315015c2bd93e094e".parse().unwrap(),
        width: None,
        height: None,
        url: None,
    };

    let repo = CachingUserInfoRepository::new(store().await?, Arc::new(MockUserInfoService::new()));
    repo.set_avatar_metadata(
        &account_id!("user@prose.org"),
        &user_id!("a@prose.org"),
        Some(&metadata),
    )
    .await?;

    let expected_user_info = UserInfo {
        avatar: Some(Avatar {
            id: "fa3c5706e27f6a0093981bb315015c2bd93e094e".parse().unwrap(),
            source: AvatarSource::Pep {
                mime_type: "image/jpg".to_string(),
            },
            owner: user_id!("a@prose.org").into(),
        }),
        activity: None,
        availability: Availability::Unavailable,
    };

    assert_eq!(
        repo.get_user_info(&account_id!("user@prose.org"), &user_id!("a@prose.org"))
            .await?
            .as_ref(),
        Some(&expected_user_info)
    );

    Ok(())
}

#[async_test]
async fn test_persists_metadata_and_user_activity() -> Result<()> {
    let metadata = AvatarMetadata {
        bytes: 1001,
        mime_type: "image/jpg".to_string(),
        checksum: "fa3c5706e27f6a0093981bb315015c2bd93e094e".parse().unwrap(),
        width: None,
        height: None,
        url: None,
    };

    let activity = UserStatus {
        emoji: "🍕".to_string(),
        status: Some("Eating pizza".to_string()),
    };

    let store = store().await?;

    let repo = CachingUserInfoRepository::new(store.clone(), Arc::new(MockUserInfoService::new()));
    repo.set_avatar_metadata(
        &account_id!("user@prose.org"),
        &user_id!("a@prose.org"),
        Some(&metadata),
    )
    .await?;
    repo.set_user_activity(
        &account_id!("user@prose.org"),
        &user_id!("a@prose.org"),
        Some(&activity),
    )
    .await?;

    let expected_user_info = UserInfo {
        avatar: Some(Avatar {
            id: "fa3c5706e27f6a0093981bb315015c2bd93e094e".parse().unwrap(),
            source: AvatarSource::Pep {
                mime_type: "image/jpg".to_string(),
            },
            owner: user_id!("a@prose.org").into(),
        }),
        activity: Some(activity),
        availability: Availability::Unavailable,
    };

    assert_eq!(
        repo.get_user_info(&account_id!("user@prose.org"), &user_id!("a@prose.org"))
            .await?
            .as_ref(),
        Some(&expected_user_info)
    );

    let repo = CachingUserInfoRepository::new(store, Arc::new(MockUserInfoService::new()));
    assert_eq!(
        repo.get_user_info(&account_id!("user@prose.org"), &user_id!("a@prose.org"))
            .await?
            .as_ref(),
        Some(&expected_user_info)
    );

    Ok(())
}

#[async_test]
async fn test_does_not_persist_availability() -> Result<()> {
    let store = store().await?;

    let mut service = MockUserInfoService::new();
    service
        .expect_load_latest_avatar_metadata()
        .times(2)
        .returning(|_| Box::pin(async { Ok(None) }));
    let service = Arc::new(service);

    let repo = CachingUserInfoRepository::new(store.clone(), service.clone());
    repo.set_user_presence(
        &account_id!("user@prose.org"),
        &user_resource_id!("a@prose.org/a").into(),
        &Presence {
            priority: 1,
            availability: Availability::Available,
            status: None,
        },
    )
    .await?;

    let mut expected_user_info = UserInfo {
        avatar: None,
        activity: None,
        availability: Availability::Available,
    };

    assert_eq!(
        repo.get_user_info(&account_id!("user@prose.org"), &user_id!("a@prose.org"))
            .await?
            .as_ref(),
        Some(&expected_user_info)
    );

    let repo = CachingUserInfoRepository::new(store.clone(), service);

    expected_user_info.availability = Availability::Unavailable;
    assert_eq!(
        repo.get_user_info(&account_id!("user@prose.org"), &user_id!("a@prose.org"))
            .await?
            .as_ref(),
        Some(&expected_user_info)
    );

    Ok(())
}

#[async_test]
async fn test_uses_highest_presence() -> Result<()> {
    let mut service = MockUserInfoService::new();
    service
        .expect_load_latest_avatar_metadata()
        .return_once(|_| Box::pin(async { Ok(None) }));

    let repo = CachingUserInfoRepository::new(store().await?, Arc::new(service));

    repo.set_user_presence(
        &account_id!("user@prose.org"),
        &user_resource_id!("a@prose.org/b").into(),
        &Presence {
            priority: 2,
            availability: Availability::Away,
            status: None,
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
        },
    )
    .await?;

    assert_eq!(
        repo.resolve_user_id(&account_id!("user@prose.org"), &user_id!("a@prose.org")),
        user_resource_id!("a@prose.org/b").into()
    );
    assert_eq!(
        repo.get_user_info(&account_id!("user@prose.org"), &user_id!("a@prose.org"))
            .await?
            .as_ref(),
        Some(&UserInfo {
            avatar: None,
            activity: None,
            availability: Availability::Away,
        })
    );

    Ok(())
}
