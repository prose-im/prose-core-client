// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use chrono::{TimeZone, Utc};
use mockall::predicate;

use prose_core_client::domain::shared::models::{UserId, UserResourceId};
use prose_core_client::domain::user_info::services::impls::UserInfoDomainService;
use prose_core_client::domain::user_info::services::UserInfoDomainService as UserInfoDomainServiceTrait;
use prose_core_client::test::{ConstantTimeProvider, MockUserInfoDomainServiceDependencies};
use prose_core_client::{user_id, user_resource_id};

#[tokio::test]
async fn test_load_user_metadata_resolves_full_jid() -> Result<()> {
    let mut deps = MockUserInfoDomainServiceDependencies::default();

    deps.time_provider = Arc::new(ConstantTimeProvider::ymd(2023, 09, 11));

    deps.user_info_repo
        .expect_resolve_user_id()
        .once()
        .with(
            predicate::always(),
            predicate::eq(user_id!("request@prose.org")),
        )
        .return_once(|_, _| Some(user_resource_id!("request@prose.org/resource")));

    deps.user_profile_service
        .expect_load_user_metadata()
        .once()
        .with(
            predicate::eq(user_resource_id!("request@prose.org/resource")),
            predicate::eq(Utc.with_ymd_and_hms(2023, 09, 11, 0, 0, 0).unwrap()),
        )
        .return_once(|_, _| Box::pin(async { Ok(None) }));

    let service = UserInfoDomainService::from(deps.into_deps());
    service
        .get_user_metadata(&user_id!("request@prose.org"))
        .await?;

    Ok(())
}
