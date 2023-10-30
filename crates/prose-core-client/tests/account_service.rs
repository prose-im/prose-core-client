// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use mockall::predicate;

use prose_core_client::domain::settings::models::AccountSettings;
use prose_core_client::dtos::Availability;
use prose_core_client::services::AccountService;
use prose_core_client::test::{mock_data, MockAppDependencies};

#[tokio::test]
async fn test_set_availability_updates_settings() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.user_account_service
        .expect_set_availability()
        .once()
        .with(predicate::always(), predicate::eq(Availability::Away))
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.account_settings_repo
        .expect_update()
        .once()
        .with(
            predicate::eq(mock_data::account_jid().into_bare()),
            predicate::always(),
        )
        .return_once(|_, f| {
            Box::pin(async {
                let mut settings = AccountSettings::default();
                f(&mut settings);
                assert_eq!(settings.availability, Some(Availability::Away));
                Ok(())
            })
        });

    let service = AccountService::from(&deps.into_deps());
    service.set_availability(Availability::Away).await?;

    Ok(())
}
