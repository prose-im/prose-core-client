// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{bail, Result};
use insta::assert_snapshot;
use prose_core_client::test::{ClientTestAdditions, ConnectedClient, ConstantTimeProvider};
use prose_core_client::Client;
use prose_xmpp::jid_str;
use prose_xmpp::stanza::LastActivityRequest;
use xmpp_parsers::disco::{DiscoInfoQuery, DiscoInfoResult};
use xmpp_parsers::iq::{Iq, IqType};
use xmpp_parsers::ping::Ping;
use xmpp_parsers::time::TimeQuery;
use xmpp_parsers::version::VersionQuery;

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

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_handles_entity_time_query() -> Result<()> {
    let ConnectedClient { connection, .. } =
        Client::connected_client_with_time_provider(ConstantTimeProvider::ymd(2023, 08, 15))
            .await?;

    connection
        .receive_stanza(Iq::from_get("req-id", TimeQuery).with_from(jid_str!("prose.org")))
        .await;

    let sent_stanzas = connection.sent_stanza_strings();
    assert_eq!(sent_stanzas.len(), 1);
    assert_snapshot!(sent_stanzas[0], @r###"
        <iq xmlns='jabber:client' id="req-id" to="prose.org" type="result"><time xmlns='urn:xmpp:time'><tzo>+00:00</tzo><utc>2023-08-15T00:00:00Z</utc></time></iq>
    "###);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_handles_software_version_query() -> Result<()> {
    let ConnectedClient { connection, .. } = Client::connected_client().await?;

    connection
        .receive_stanza(
            Iq::from_get("req-id", VersionQuery).with_from(jid_str!("client@prose.org")),
        )
        .await;

    let sent_stanzas = connection.sent_stanza_strings();
    assert_eq!(sent_stanzas.len(), 1);
    assert_snapshot!(sent_stanzas[0], @r###"
        <iq xmlns='jabber:client' id="req-id" to="client@prose.org" type="result"><query xmlns='jabber:iq:version'><name>prose-test-client</name><version>1.2.3</version><os>unknown os</os></query></iq>
    "###);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_handles_disco_request() -> Result<()> {
    let ConnectedClient { connection, .. } = Client::connected_client().await?;

    connection
        .receive_stanza(
            Iq::from_get("req-id", DiscoInfoQuery { node: None })
                .with_from(jid_str!("client@prose.org")),
        )
        .await;

    let sent_stanzas = connection.sent_stanzas();
    assert_eq!(sent_stanzas.len(), 1);

    // Let's not look too deep into the stanza since the actual features are subject to change.
    // We only make sure that this is indeed a disco response and that it has at least one feature.

    let iq = Iq::try_from(sent_stanzas[0].clone())?;
    let IqType::Result(Some(payload)) = iq.payload else {
        bail!("Invalid iq or missing payload")
    };

    let request = DiscoInfoResult::try_from(payload)?;
    assert!(request.features.len() > 0);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_handles_last_activity_request() -> Result<()> {
    let ConnectedClient { connection, .. } = Client::connected_client().await?;

    connection
        .receive_stanza(
            Iq::from_get("req-id", LastActivityRequest).with_from(jid_str!("client@prose.org")),
        )
        .await;

    let sent_stanzas = connection.sent_stanza_strings();
    assert_eq!(sent_stanzas.len(), 1);
    assert_snapshot!(sent_stanzas[0], @r###"
        <iq xmlns='jabber:client' id="req-id" to="client@prose.org" type="result"><query xmlns='jabber:iq:last' seconds="0"/></iq>
    "###);

    Ok(())
}
