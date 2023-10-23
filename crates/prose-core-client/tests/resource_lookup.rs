// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use insta::assert_snapshot;
use prose_core_client::Client;
use prose_xmpp::jid;
use xmpp_parsers::iq::Iq;
use xmpp_parsers::presence::Presence;
use xmpp_parsers::stanza_error::{DefinedCondition, ErrorType, StanzaError};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_resolves_bare_jid_to_full() -> Result<()> {
    panic!("FIXME")
    // let ConnectedClient {
    //     client,
    //     connection,
    //     id_provider,
    //     ..
    // } = Client::connected_client().await?;
    //
    // let jid = jid!("a@prose.org").to_bare();
    //
    // // We'll just send an error since we're not interested in the result, just the sent stanza
    // {
    //     connection.set_stanza_handler(move |_| {
    //         vec![Iq::from_error(
    //             "id-1",
    //             StanzaError::new(
    //                 ErrorType::Cancel,
    //                 DefinedCondition::NotAllowed,
    //                 "en",
    //                 "Something went wrong",
    //             ),
    //         )
    //         .into()]
    //     });
    // }
    //
    // _ = client.load_user_metadata(&jid).await;
    //
    // let sent_stanzas = connection.sent_stanza_strings();
    // assert_eq!(sent_stanzas.len(), 1);
    // assert_snapshot!(sent_stanzas[0], @r###"
    //     <iq xmlns='jabber:client' id="id-1" to="a@prose.org" type="get"><time xmlns='urn:xmpp:time'/></iq>
    // "###);
    //
    // connection
    //     .receive_stanza(
    //         Presence::available()
    //             .with_from(jid!("a@prose.org/r1"))
    //             .with_priority(1),
    //     )
    //     .await;
    //
    // connection.reset();
    // id_provider.reset();
    //
    // _ = client.load_user_metadata(&jid).await;
    //
    // let sent_stanzas = connection.sent_stanza_strings();
    // assert_eq!(sent_stanzas.len(), 1);
    // assert_snapshot!(sent_stanzas[0], @r###"
    //     <iq xmlns='jabber:client' id="id-1" to="a@prose.org/r1" type="get"><time xmlns='urn:xmpp:time'/></iq>
    // "###);
    //
    // connection
    //     .receive_stanza(
    //         Presence::available()
    //             .with_from(jid!("a@prose.org/r2"))
    //             .with_priority(2),
    //     )
    //     .await;
    //
    // connection.reset();
    // id_provider.reset();
    //
    // _ = client.load_user_metadata(&jid).await;
    //
    // let sent_stanzas = connection.sent_stanza_strings();
    // assert_eq!(sent_stanzas.len(), 1);
    // assert_snapshot!(sent_stanzas[0], @r###"
    //     <iq xmlns='jabber:client' id="id-1" to="a@prose.org/r2" type="get"><time xmlns='urn:xmpp:time'/></iq>
    // "###);
    //
    // Ok(())
}
