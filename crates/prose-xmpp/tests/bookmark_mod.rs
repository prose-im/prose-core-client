// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use anyhow::Result;
use insta::assert_snapshot;
use minidom::Element;
use xmpp_parsers::bookmarks2::{Autojoin, Conference};
use xmpp_parsers::iq::Iq;
use xmpp_parsers::pubsub;

use prose_xmpp::stanza::ConferenceBookmark;
use prose_xmpp::test::{ClientTestAdditions, ConnectedClient};
use prose_xmpp::{jid_str, mods, Client, Event};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_loads_bookmarks() -> Result<()> {
    let ConnectedClient {
        connection, client, ..
    } = Client::connected_client().await?;

    let xml = r#"<iq type='result' to='juliet@capulet.lit/balcony' id='id-1' xmlns='jabber:client'>
  <pubsub xmlns='http://jabber.org/protocol/pubsub'>
    <items node='urn:xmpp:bookmarks:1'>
      <item id='theplay@conference.shakespeare.lit'>
        <conference xmlns='urn:xmpp:bookmarks:1' name='The Play&apos;s the Thing' autojoin='true'>
          <nick>JC</nick>
        </conference>
      </item>
      <item id='orchard@conference.shakespeare.lit'>
        <conference xmlns='urn:xmpp:bookmarks:1' name='The Orcard' autojoin='1'>
          <nick>JC</nick>
          <extensions>
            <state xmlns='http://myclient.example/bookmark/state' minimized='true' />
          </extensions>
        </conference>
      </item>
    </items>
  </pubsub>
</iq>
    "#;

    connection.set_stanza_handler(|_| vec![Element::from_str(xml).unwrap()]);

    let bookmark = client.get_mod::<mods::Bookmark2>();
    let bookmarks = bookmark.load_bookmarks().await?;

    let sent_stanzas = connection.sent_stanza_strings();
    assert_eq!(sent_stanzas.len(), 1);
    assert_snapshot!(sent_stanzas[0], @r###"
        <iq xmlns='jabber:client' id="id-1" type="get"><pubsub xmlns='http://jabber.org/protocol/pubsub'><items node="urn:xmpp:bookmarks:1"/></pubsub></iq>
    "###);

    assert_eq!(
        bookmarks,
        vec![
            ConferenceBookmark {
                jid: jid_str!("theplay@conference.shakespeare.lit"),
                conference: Conference {
                    autojoin: Autojoin::True,
                    name: Some("The Play's the Thing".to_string()),
                    nick: Some("JC".to_string()),
                    password: None,
                    extensions: vec![],
                }
            },
            ConferenceBookmark {
                jid: jid_str!("orchard@conference.shakespeare.lit"),
                conference: Conference {
                    autojoin: Autojoin::True,
                    name: Some("The Orcard".to_string()),
                    nick: Some("JC".to_string()),
                    password: None,
                    extensions: vec![Element::builder(
                        "state",
                        "http://myclient.example/bookmark/state"
                    )
                    .attr("minimized", "true")
                    .build()],
                }
            }
        ]
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_loads_legacy_bookmarks() -> Result<()> {
    let ConnectedClient {
        connection, client, ..
    } = Client::connected_client().await?;

    let xml = r#"<iq type='result' to='juliet@capulet.lit/randomID' id='id-1' xmlns='jabber:client'>
  <pubsub xmlns='http://jabber.org/protocol/pubsub'>
    <items node='storage:bookmarks'>
      <item id='current'>
        <storage xmlns='storage:bookmarks'>
          <conference name='The Play&apos;s the Thing' autojoin='true' jid='theplay@conference.shakespeare.lit'>
            <nick>JC</nick>
          </conference>
        </storage>
      </item>
    </items>
  </pubsub>
</iq>
    "#;

    connection.set_stanza_handler(|_| vec![Element::from_str(xml).unwrap()]);

    let bookmark = client.get_mod::<mods::Bookmark>();
    let bookmarks = bookmark.load_bookmarks().await?;

    let sent_stanzas = connection.sent_stanza_strings();
    assert_eq!(sent_stanzas.len(), 1);
    assert_snapshot!(sent_stanzas[0], @r###"
        <iq xmlns='jabber:client' id="id-1" type="get"><pubsub xmlns='http://jabber.org/protocol/pubsub'><items node="storage:bookmarks"/></pubsub></iq>
    "###);

    assert_eq!(
        bookmarks,
        vec![ConferenceBookmark {
            jid: jid_str!("theplay@conference.shakespeare.lit"),
            conference: Conference {
                autojoin: Autojoin::True,
                name: Some("The Play's the Thing".to_string()),
                nick: Some("JC".to_string()),
                password: None,
                extensions: vec![],
            }
        },]
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_publishes_bookmark() -> Result<()> {
    let ConnectedClient {
        connection, client, ..
    } = Client::connected_client().await?;

    let bookmark = client.get_mod::<mods::Bookmark2>();

    connection.set_stanza_handler(|_| vec![Iq::from_result("id-1", None::<pubsub::PubSub>).into()]);

    bookmark
        .publish_bookmark(
            jid_str!("room@prose.org"),
            Conference {
                autojoin: Autojoin::True,
                name: Some("Room Name".to_string()),
                nick: Some("User Nick".to_string()),
                password: Some("Room password".to_string()),
                extensions: vec![],
            },
        )
        .await?;

    let sent_stanzas = connection.sent_stanza_strings();
    assert_eq!(sent_stanzas.len(), 1);
    assert_snapshot!(sent_stanzas[0]);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_publishes_legacy_bookmarks() -> Result<()> {
    let ConnectedClient {
        connection, client, ..
    } = Client::connected_client().await?;

    let bookmark = client.get_mod::<mods::Bookmark>();

    connection.set_stanza_handler(|_| vec![Iq::from_result("id-1", None::<pubsub::PubSub>).into()]);

    bookmark
        .publish_bookmarks(vec![ConferenceBookmark {
            jid: jid_str!("room@prose.org"),
            conference: Conference {
                autojoin: Autojoin::True,
                name: Some("Room Name".to_string()),
                nick: Some("User Nick".to_string()),
                password: Some("Room password".to_string()),
                extensions: vec![],
            },
        }])
        .await?;

    let sent_stanzas = connection.sent_stanza_strings();
    assert_eq!(sent_stanzas.len(), 1);
    assert_snapshot!(sent_stanzas[0]);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_retracts_bookmark() -> Result<()> {
    let ConnectedClient {
        connection, client, ..
    } = Client::connected_client().await?;

    let bookmark = client.get_mod::<mods::Bookmark2>();

    connection.set_stanza_handler(|_| vec![Iq::from_result("id-1", None::<pubsub::PubSub>).into()]);

    bookmark
        .retract_bookmark(jid_str!("room@prose.org"))
        .await?;

    let sent_stanzas = connection.sent_stanza_strings();
    assert_eq!(sent_stanzas.len(), 1);
    assert_snapshot!(sent_stanzas[0], @r###"
        <iq xmlns='jabber:client' id="id-1" type="set"><pubsub xmlns='http://jabber.org/protocol/pubsub'><retract node="urn:xmpp:bookmarks:1" notify="true"><item id="room@prose.org"/></retract></pubsub></iq>
    "###);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_bookmarks_published_event() -> Result<()> {
    let client = Client::connected_client().await?;

    let xml = r#"<message from='test@prose.org' to='test@prose.org/test' type='headline' id='new-room1' xmlns='jabber:client'>
  <event xmlns='http://jabber.org/protocol/pubsub#event'>
    <items node='urn:xmpp:bookmarks:1'>
      <item id='theplay@conference.shakespeare.lit'>
        <conference xmlns='urn:xmpp:bookmarks:1' name='The Play&apos;s the Thing' autojoin='1'>
          <nick>JC</nick>
        </conference>
      </item>
    </items>
  </event>
</message>
    "#;

    client
        .connection
        .receive_stanza(Element::from_str(xml).unwrap())
        .await;

    let sent_events = client.sent_events();
    assert_eq!(sent_events.len(), 1);
    assert_eq!(
        sent_events[0],
        Event::Bookmark2(mods::bookmark2::Event::BookmarksPublished {
            bookmarks: vec![ConferenceBookmark {
                jid: jid_str!("theplay@conference.shakespeare.lit"),
                conference: Conference {
                    autojoin: Autojoin::True,
                    name: Some("The Play's the Thing".to_string()),
                    nick: Some("JC".to_string()),
                    password: None,
                    extensions: vec![],
                }
            }]
        })
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_legacy_bookmark_event() -> Result<()> {
    let client = Client::connected_client().await?;

    let xml = r#"<message from='juliet@capulet.lit' to='juliet@capulet.lit/balcony' type='headline' id='rnfoo1' xmlns="jabber:client">
  <event xmlns='http://jabber.org/protocol/pubsub#event'>
    <items node='storage:bookmarks'>
      <item id='current'>
        <storage xmlns='storage:bookmarks'>
          <conference name='The Play&apos;s the Thing' autojoin='true' jid='theplay@conference.shakespeare.lit'>
            <nick>JC</nick>
          </conference>
        </storage>
      </item>
    </items>
  </event>
</message>
    "#;

    client
        .connection
        .receive_stanza(Element::from_str(xml).unwrap())
        .await;

    let sent_events = client.sent_events();
    assert_eq!(sent_events.len(), 1);
    assert_eq!(
        sent_events[0],
        Event::Bookmark(mods::bookmark::Event::BookmarksChanged {
            bookmarks: vec![ConferenceBookmark {
                jid: jid_str!("theplay@conference.shakespeare.lit"),
                conference: Conference {
                    autojoin: Autojoin::True,
                    name: Some("The Play's the Thing".to_string()),
                    nick: Some("JC".to_string()),
                    password: None,
                    extensions: vec![],
                }
            }]
        })
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_bookmarks_retracted_event() -> Result<()> {
    let client = Client::connected_client().await?;

    let xml = r#"<message from='juliet@capulet.lit' to='juliet@capulet.lit/balcony' type='headline' id='removed-room1' xmlns='jabber:client'>
  <event xmlns='http://jabber.org/protocol/pubsub#event'>
    <items node='urn:xmpp:bookmarks:1'>
      <retract id='theplay@conference.shakespeare.lit'/>
    </items>
  </event>
</message>
    "#;

    client
        .connection
        .receive_stanza(Element::from_str(xml).unwrap())
        .await;

    let sent_events = client.sent_events();
    assert_eq!(sent_events.len(), 1);
    assert_eq!(
        sent_events[0],
        Event::Bookmark2(mods::bookmark2::Event::BookmarksRetracted {
            jids: vec![jid_str!("theplay@conference.shakespeare.lit")]
        })
    );

    Ok(())
}
