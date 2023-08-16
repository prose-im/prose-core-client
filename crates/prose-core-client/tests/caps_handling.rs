// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use insta::assert_snapshot;
use jid::{BareJid, FullJid};
use prose_core_client::avatar_cache::NoopAvatarCache;
use prose_core_client::data_cache::sqlite::SQLiteCache;
use prose_core_client::test::{BareJidTestAdditions, IncrementingIDProvider};
use prose_core_client::types::{Availability, SoftwareVersion};
use prose_core_client::ClientBuilder;
use prose_xmpp::{test, SystemTimeProvider};
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_start_sequence() -> Result<()> {
    let connection = Arc::new(test::Connection::default());
    let data_cache = Arc::new(SQLiteCache::in_memory_cache());

    let client = ClientBuilder::new()
        .set_connector_provider(test::Connector::provider(connection.clone()))
        .set_data_cache(data_cache.clone())
        .set_avatar_cache(NoopAvatarCache::default())
        .set_id_provider(IncrementingIDProvider::new())
        .set_time_provider(SystemTimeProvider::default())
        .set_software_version(SoftwareVersion {
            name: "prose-test-client".to_string(),
            version: "1.2.3".to_string(),
            os: Some("unknown os".to_string()),
        })
        .build();

    client
        .connect(
            &FullJid::from_str(&format!("{}/test", BareJid::ours()))?,
            "",
            Availability::Available,
        )
        .await?;

    assert_snapshot!(connection.sent_stanza_strings().join("\n\n"));

    Ok(())
}
