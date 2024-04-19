// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::dtos::{MucId, UserId};
use prose_core_client::{muc_id, user_id};
use prose_proc_macros::mt_test;

use super::helpers::TestClient;

#[mt_test]
async fn test_joins_room() -> Result<()> {
    let client = TestClient::new().await;

    client
        .perform_login(user_id!("user@prose.org"), "secret")
        .await?;

    client
        .perform_join_room(muc_id!("room@conference.prose.org"))
        .await?;

    Ok(())
}
