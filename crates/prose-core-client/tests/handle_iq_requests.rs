// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use insta::assert_snapshot;
use prose_core_client::test::{ClientTestAdditions, ConnectedClient};
use prose_core_client::{jid_str, Client};
use xmpp_parsers::iq::Iq;
use xmpp_parsers::ping::Ping;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_handles_ping() -> Result<()> {
    let ConnectedClient { connection, .. } = Client::connected_client().await?;

    // Simulate receiving a ping request
    connection
        .receive_stanza(Iq::from_get("req-id", Ping).with_from(jid_str!("prose.org")))
        .await;

    let sent_stanzas = connection.sent_stanza_strings();
    assert_eq!(sent_stanzas.len(), 1);
    assert_snapshot!(sent_stanzas[0], @r###"
        <iq xmlns='jabber:client' id="req-id" to="prose.org" type="result"/>
    "###);

    Ok(())
}
