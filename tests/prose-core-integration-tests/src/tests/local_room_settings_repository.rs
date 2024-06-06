// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use chrono::{TimeZone, Utc};
use pretty_assertions::assert_eq;

use prose_core_client::domain::settings::models::LocalRoomSettings;
use prose_core_client::domain::settings::repos::LocalRoomSettingsRepository as LocalRoomSettingsRepositoryTrait;
use prose_core_client::domain::shared::models::{AccountId, UserId};
use prose_core_client::infra::settings::LocalRoomSettingsRepository;
use prose_core_client::{account_id, user_id};

use crate::tests::{async_test, store};

#[async_test]
async fn test_save_and_load_local_room_settings() -> Result<()> {
    let repo = LocalRoomSettingsRepository::new(store().await?);

    assert_eq!(
        repo.get(
            &account_id!("a@prose.org"),
            &user_id!("room1@prose.org").into()
        )
        .await?,
        LocalRoomSettings::default()
    );

    repo.update(
        &account_id!("a@prose.org"),
        &user_id!("room1@prose.org").into(),
        Box::new(|settings: &mut LocalRoomSettings| {
            settings.last_catchup_time =
                Some(Utc.with_ymd_and_hms(2024, 05, 14, 12, 00, 00).unwrap());
        }),
    )
    .await?;

    repo.update(
        &account_id!("a@prose.org"),
        &user_id!("room2@prose.org").into(),
        Box::new(|settings: &mut LocalRoomSettings| {
            settings.last_catchup_time =
                Some(Utc.with_ymd_and_hms(2024, 05, 14, 11, 00, 00).unwrap());
        }),
    )
    .await?;

    repo.update(
        &account_id!("b@prose.org"),
        &user_id!("room1@prose.org").into(),
        Box::new(|settings: &mut LocalRoomSettings| {
            settings.last_catchup_time =
                Some(Utc.with_ymd_and_hms(2024, 05, 14, 10, 00, 00).unwrap());
        }),
    )
    .await?;

    assert_eq!(
        repo.get(
            &account_id!("a@prose.org"),
            &user_id!("room1@prose.org").into()
        )
        .await?
        .last_catchup_time,
        Some(Utc.with_ymd_and_hms(2024, 05, 14, 12, 00, 00).unwrap())
    );
    assert_eq!(
        repo.get(
            &account_id!("a@prose.org"),
            &user_id!("room2@prose.org").into()
        )
        .await?
        .last_catchup_time,
        Some(Utc.with_ymd_and_hms(2024, 05, 14, 11, 00, 00).unwrap())
    );

    repo.clear_cache(&account_id!("a@prose.org")).await?;

    assert_eq!(
        repo.get(
            &account_id!("a@prose.org"),
            &user_id!("room1@prose.org").into()
        )
        .await?
        .last_catchup_time,
        None
    );
    assert_eq!(
        repo.get(
            &account_id!("a@prose.org"),
            &user_id!("room2@prose.org").into()
        )
        .await?
        .last_catchup_time,
        None
    );
    assert_eq!(
        repo.get(
            &account_id!("b@prose.org"),
            &user_id!("room1@prose.org").into()
        )
        .await?
        .last_catchup_time,
        Some(Utc.with_ymd_and_hms(2024, 05, 14, 10, 00, 00).unwrap())
    );

    Ok(())
}
