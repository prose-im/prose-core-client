// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::domain::messaging::repos::DraftsRepository as DomainDraftsRepository;
use prose_core_client::infra::messaging::DraftsRepository;
use prose_xmpp::bare;

use crate::tests::{async_test, store};

#[async_test]
async fn test_saves_and_loads_draft() -> Result<()> {
    let repo = DraftsRepository::new(store().await?);

    let jid_a = bare!("a@prose.org");
    let jid_b = bare!("b@prose.org");

    assert_eq!(repo.get(&jid_a).await?, None);
    assert_eq!(repo.get(&jid_b).await?, None);

    repo.set(&jid_a, Some("Hello")).await?;
    repo.set(&jid_b, Some("World")).await?;

    assert_eq!(repo.get(&jid_a).await?, Some("Hello".to_string()));
    assert_eq!(repo.get(&jid_b).await?, Some("World".to_string()));

    repo.set(&jid_b, None).await?;

    assert_eq!(repo.get(&jid_a).await?, Some("Hello".to_string()));
    assert_eq!(repo.get(&jid_b).await?, None);

    Ok(())
}
