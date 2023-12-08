// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;

use prose_core_client::domain::contacts::models::{Contact, Group};
use prose_core_client::domain::contacts::repos::ContactsRepository;
use prose_core_client::domain::contacts::services::mocks::MockContactsService;
use prose_core_client::domain::shared::models::UserId;
use prose_core_client::infra::contacts::CachingContactsRepository;
use prose_core_client::user_id;

use crate::tests::async_test;

#[async_test]
async fn test_loads_and_caches_contacts() -> Result<()> {
    let contacts = vec![
        Contact {
            id: user_id!("a@prose.org"),
            name: None,
            group: Group::Favorite,
        },
        Contact {
            id: user_id!("b@prose.org"),
            name: None,
            group: Group::Team,
        },
    ];

    let service = {
        let contacts = contacts.clone();
        let mut service = MockContactsService::new();
        service
            .expect_load_contacts()
            .times(1)
            .return_once(|_| Box::pin(async move { Ok(contacts) }));
        service
    };

    let repo = CachingContactsRepository::new(Arc::new(service));
    assert_eq!(
        repo.get_all(&user_id!("account@prose.org")).await?,
        contacts
    );
    assert_eq!(
        repo.get_all(&user_id!("account@prose.org")).await?,
        contacts
    );

    Ok(())
}
