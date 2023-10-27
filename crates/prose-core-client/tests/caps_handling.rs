// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use insta::assert_snapshot;
use jid::{BareJid, FullJid};
//use prose_core_client::test::{ClientTestAdditions, ConnectedClient};
use prose_core_client::Client;
use prose_xmpp::test::{BareJidTestAdditions, IncrementingIDProvider};
use prose_xmpp::{test, SystemTimeProvider};
use std::str::FromStr;
use std::sync::Arc;

// Snapshots will need to be updated if/when caps features change…

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_start_sequence() -> Result<()> {
    panic!("FIXME");
    // let connection = Arc::new(test::Connection::default());
    // let data_cache = Arc::new(PlatformCache::temporary_cache().await?);
    //
    // connection.use_start_sequence_handler();
    //
    // let client = ClientBuilder::new()
    //     .set_connector_provider(test::Connector::provider(connection.clone()))
    //     .set_data_cache(data_cache.clone())
    //     .set_avatar_cache(NoopAvatarCache::default())
    //     .set_id_provider(IncrementingIDProvider::new())
    //     .set_time_provider(SystemTimeProvider::default())
    //     .set_software_version(SoftwareVersion {
    //         name: "prose-test-client".to_string(),
    //         version: "1.2.3".to_string(),
    //         os: Some("unknown os".to_string()),
    //     })
    //     .build();
    //
    // client
    //     .connect(
    //         &FullJid::from_str(&format!("{}/test", BareJid::ours()))?,
    //         "",
    //         Availability::Available,
    //     )
    //     .await?;
    //
    // assert_snapshot!(connection.sent_stanza_strings().join("\n\n"));
    //
    // Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_sends_caps_when_changing_availability() -> Result<()> {
    panic!("FIXME");
    // let ConnectedClient {
    //     client, connection, ..
    // } = Client::connected_client().await?;
    //
    // client.set_availability(Availability::DoNotDisturb).await?;
    //
    // let sent_stanzas = connection.sent_stanza_strings();
    // assert_eq!(sent_stanzas.len(), 1);
    // assert_snapshot!(sent_stanzas[0]);
    //
    // Ok(())
}
