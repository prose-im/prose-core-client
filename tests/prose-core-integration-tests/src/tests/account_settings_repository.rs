// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::domain::settings::models::AccountSettings;
use prose_core_client::domain::settings::repos::AccountSettingsRepository as DomainAccountSettingsRepository;
use prose_core_client::domain::shared::models::Availability;
use prose_core_client::infra::settings::AccountSettingsRepository;
use prose_xmpp::bare;

use crate::tests::{async_test, store};

#[async_test]
async fn test_save_and_load_account_settings() -> Result<()> {
    let repo = AccountSettingsRepository {
        store: store().await?,
    };

    assert_eq!(
        repo.get(&bare!("a@prose.org")).await?,
        AccountSettings::default()
    );

    repo.update(
        &bare!("a@prose.org"),
        Box::new(|settings: &mut AccountSettings| {
            settings.availability = Availability::Away;
        }),
    )
    .await?;

    let expected_settings = AccountSettings {
        availability: Availability::Away,
    };
    assert_ne!(expected_settings, AccountSettings::default());

    assert_eq!(repo.get(&bare!("a@prose.org")).await?, expected_settings);
    assert_eq!(
        repo.get(&bare!("b@prose.org")).await?,
        AccountSettings::default()
    );

    Ok(())
}
