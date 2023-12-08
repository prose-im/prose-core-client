// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::domain::settings::models::AccountSettings;
use prose_core_client::domain::settings::repos::AccountSettingsRepository as DomainAccountSettingsRepository;
use prose_core_client::domain::shared::models::{Availability, UserId};
use prose_core_client::infra::settings::AccountSettingsRepository;
use prose_core_client::user_id;

use crate::tests::{async_test, store};

#[async_test]
async fn test_save_and_load_account_settings() -> Result<()> {
    let repo = AccountSettingsRepository::new(store().await?);

    assert_eq!(
        repo.get(&user_id!("a@prose.org")).await?,
        AccountSettings::default()
    );

    repo.update(
        &user_id!("a@prose.org"),
        Box::new(|settings: &mut AccountSettings| {
            settings.availability = Some(Availability::Away);
        }),
    )
    .await?;

    let expected_settings = AccountSettings {
        availability: Some(Availability::Away),
        resource: None,
    };
    assert_ne!(expected_settings, AccountSettings::default());

    assert_eq!(repo.get(&user_id!("a@prose.org")).await?, expected_settings);
    assert_eq!(
        repo.get(&user_id!("b@prose.org")).await?,
        AccountSettings::default()
    );

    Ok(())
}
