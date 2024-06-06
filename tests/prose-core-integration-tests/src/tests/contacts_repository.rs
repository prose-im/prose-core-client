// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;

use prose_core_client::domain::contacts::models::Contact;
use prose_core_client::domain::contacts::repos::ContactListRepository;
use prose_core_client::domain::contacts::services::mocks::MockContactListService;
use prose_core_client::domain::shared::models::{AccountId, UserId};
use prose_core_client::dtos::PresenceSubscription;
use prose_core_client::infra::contacts::CachingContactsRepository;
use prose_core_client::{account_id, user_id};

use crate::tests::async_test;

#[async_test]
async fn test_loads_and_caches_contacts() -> Result<()> {
    let contacts = vec![
        Contact {
            id: user_id!("a@prose.org"),
            presence_subscription: PresenceSubscription::Requested,
        },
        Contact {
            id: user_id!("b@prose.org"),
            presence_subscription: PresenceSubscription::Requested,
        },
    ];

    let service = {
        let contacts = contacts.clone();
        let mut service = MockContactListService::new();
        service
            .expect_load_contacts()
            .times(1)
            .return_once(|| Box::pin(async move { Ok(contacts) }));
        service
    };

    let repo = CachingContactsRepository::new(Arc::new(service));
    assert_eq!(
        repo.get_all(&account_id!("user@prose.org")).await?,
        contacts
    );
    assert_eq!(
        repo.get_all(&account_id!("user@prose.org")).await?,
        contacts
    );

    Ok(())
}
