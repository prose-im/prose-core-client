// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use insta::assert_snapshot;
use jid::BareJid;
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::message::MessageType;

use prose_xmpp::stanza::message::mam::ArchivedMessage;
use prose_xmpp::stanza::message::{carbons, Forwarded};
use prose_xmpp::stanza::Message;
use prose_xmpp::test::{BareJidTestAdditions, ClientTestAdditions, ConnectedClient};
use prose_xmpp::{bare, jid, mods, Client, Event};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_sends_message_event() -> Result<()> {
    let ConnectedClient {
        connection,
        sent_events,
        ..
    } = Client::connected_client().await?;

    let message = Message::new()
        .set_from(jid!("sender@prose.org"))
        .set_body("Hello World");

    connection.receive_stanza(message.clone()).await;
    assert_eq!(
        *sent_events.read(),
        vec![Event::Chat(mods::chat::Event::Message(message))]
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_does_not_send_message_event_for_archived_message() -> Result<()> {
    let ConnectedClient {
        connection,
        sent_events,
        ..
    } = Client::connected_client().await?;

    let message = Message::new()
        .set_from(jid!("sender@prose.org"))
        .set_body("Hello World")
        .set_archived_message(ArchivedMessage {
            id: "msg-id".into(),
            query_id: None,
            forwarded: Forwarded {
                delay: None,
                message: None,
            },
        });

    connection.receive_stanza(message.clone()).await;
    assert_eq!(*sent_events.read(), vec![]);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_sends_received_carbon_event() -> Result<()> {
    let ConnectedClient {
        connection,
        sent_events,
        ..
    } = Client::connected_client().await?;

    let carbon = carbons::Received {
        forwarded: Forwarded {
            delay: None,
            message: Box::new(Message::new().set_id("nested-msg-id".into())),
        },
    };

    let message = Message::new()
        .set_from(BareJid::ours())
        .set_body("Hello World")
        .set_received_carbon(carbon.clone());

    connection.receive_stanza(message.clone()).await;
    assert_eq!(
        *sent_events.read(),
        vec![Event::Chat(mods::chat::Event::Carbon(
            mods::chat::Carbon::Received(carbon.forwarded)
        ))]
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_does_not_send_received_carbon_event_for_different_user() -> Result<()> {
    let ConnectedClient {
        connection,
        sent_events,
        ..
    } = Client::connected_client().await?;

    let carbon = carbons::Received {
        forwarded: Forwarded {
            delay: None,
            message: Box::new(Message::new().set_id("nested-msg-id".into())),
        },
    };

    let message = Message::new()
        .set_from(bare!("spoof@prose.org"))
        .set_body("Hello World")
        .set_received_carbon(carbon.clone());

    connection.receive_stanza(message.clone()).await;
    assert_eq!(*sent_events.read(), vec![]);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_sends_sent_carbon_event() -> Result<()> {
    let ConnectedClient {
        connection,
        sent_events,
        ..
    } = Client::connected_client().await?;

    let carbon = carbons::Sent {
        forwarded: Forwarded {
            delay: None,
            message: Box::new(Message::new().set_id("nested-msg-id".into())),
        },
    };

    let message = Message::new()
        .set_from(BareJid::ours())
        .set_body("Hello World")
        .set_sent_carbon(carbon.clone());

    connection.receive_stanza(message.clone()).await;
    assert_eq!(
        *sent_events.read(),
        vec![Event::Chat(mods::chat::Event::Carbon(
            mods::chat::Carbon::Sent(carbon.forwarded)
        ))]
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_does_not_send_sent_carbon_event_for_different_user() -> Result<()> {
    let ConnectedClient {
        connection,
        sent_events,
        ..
    } = Client::connected_client().await?;

    let carbon = carbons::Sent {
        forwarded: Forwarded {
            delay: None,
            message: Box::new(Message::new().set_id("nested-msg-id".into())),
        },
    };

    let message = Message::new()
        .set_from(bare!("spoof@prose.org"))
        .set_body("Hello World")
        .set_sent_carbon(carbon.clone());

    connection.receive_stanza(message.clone()).await;
    assert_eq!(*sent_events.read(), vec![]);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_sends_chat_state_changed_event() -> Result<()> {
    let ConnectedClient {
        connection,
        sent_events,
        ..
    } = Client::connected_client().await?;

    connection
        .receive_stanza(
            Message::new()
                .set_from(jid!("sender@prose.org"))
                .set_chat_state(Some(ChatState::Composing))
                .set_body("Hello World"),
        )
        .await;

    assert_eq!(
        *sent_events.read(),
        vec![
            Event::Chat(mods::chat::Event::ChatStateChanged {
                from: jid!("sender@prose.org"),
                chat_state: ChatState::Composing,
                message_type: Default::default()
            }),
            Event::Chat(mods::chat::Event::Message(
                // Chat state should be removed…
                Message::new()
                    .set_from(jid!("sender@prose.org"))
                    .set_chat_state(Some(ChatState::Composing))
                    .set_body("Hello World")
            ))
        ]
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_send_message() -> Result<()> {
    let ConnectedClient {
        connection, client, ..
    } = Client::connected_client().await?;

    let chat = client.get_mod::<mods::Chat>();
    chat.send_message(
        jid!("recv@prose.org"),
        "Hello World!",
        &MessageType::Chat,
        Some(ChatState::Active),
    )?;
    chat.send_message(
        jid!("recv@prose.org"),
        "Hello World!",
        &MessageType::Chat,
        None,
    )?;

    let sent_stanzas = connection.sent_stanza_strings();
    assert_eq!(sent_stanzas.len(), 2);
    assert_snapshot!(sent_stanzas[0], @r###"
        <message xmlns='jabber:client' from="test@prose.org/test" id="id-1" to="recv@prose.org" type="chat"><body>Hello World!</body><active xmlns='http://jabber.org/protocol/chatstates'/><markable xmlns='urn:xmpp:chat-markers:0'/></message>
    "###);
    assert_snapshot!(sent_stanzas[1], @r###"
        <message xmlns='jabber:client' from="test@prose.org/test" id="id-2" to="recv@prose.org" type="chat"><body>Hello World!</body><markable xmlns='urn:xmpp:chat-markers:0'/></message>
    "###);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_update_message() -> Result<()> {
    let ConnectedClient {
        connection, client, ..
    } = Client::connected_client().await?;

    let chat = client.get_mod::<mods::Chat>();
    chat.update_message(
        "msg-id".into(),
        jid!("recv@prose.org"),
        "Updated Message",
        &MessageType::Chat,
    )?;

    let sent_stanzas = connection.sent_stanza_strings();
    assert_eq!(sent_stanzas.len(), 1);
    assert_snapshot!(sent_stanzas[0], @r###"
        <message xmlns='jabber:client' from="test@prose.org/test" id="id-1" to="recv@prose.org" type="chat"><body>Updated Message</body><replace xmlns='urn:xmpp:message-correct:0' id="msg-id"/></message>
    "###);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_send_chat_state() -> Result<()> {
    let ConnectedClient {
        connection, client, ..
    } = Client::connected_client().await?;

    let chat = client.get_mod::<mods::Chat>();
    chat.send_chat_state(
        jid!("recv@prose.org"),
        ChatState::Composing,
        &MessageType::Groupchat,
    )?;

    let sent_stanzas = connection.sent_stanza_strings();
    assert_eq!(sent_stanzas.len(), 1);
    assert_snapshot!(sent_stanzas[0], @r###"
        <message xmlns='jabber:client' from="test@prose.org/test" to="recv@prose.org" type="groupchat"><composing xmlns='http://jabber.org/protocol/chatstates'/></message>
    "###);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_react_to_chat_message() -> Result<()> {
    let ConnectedClient {
        connection, client, ..
    } = Client::connected_client().await?;

    let chat = client.get_mod::<mods::Chat>();
    chat.react_to_chat_message(
        "msg-id".into(),
        jid!("recv@prose.org/resource"),
        vec!["😅".into(), "🍕".into()],
    )?;

    let sent_stanzas = connection.sent_stanza_strings();
    assert_eq!(sent_stanzas.len(), 1);
    assert_snapshot!(sent_stanzas[0], @r###"
        <message xmlns='jabber:client' from="test@prose.org/test" id="id-1" to="recv@prose.org/resource" type="chat"><reactions xmlns='urn:xmpp:reactions:0' id="msg-id"><reaction>😅</reaction><reaction>🍕</reaction></reactions><store xmlns='urn:xmpp:hints'/></message>
    "###);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_retract_message() -> Result<()> {
    let ConnectedClient {
        connection, client, ..
    } = Client::connected_client().await?;

    let chat = client.get_mod::<mods::Chat>();
    chat.retract_message("msg-id".into(), jid!("recv@prose.org"), &MessageType::Chat)?;

    let sent_stanzas = connection.sent_stanza_strings();
    assert_eq!(sent_stanzas.len(), 1);
    assert_snapshot!(sent_stanzas[0], @r###"
        <message xmlns='jabber:client' from="test@prose.org/test" id="id-1" to="recv@prose.org" type="chat"><body>This person attempted to retract a previous message, but it's unsupported by your client.</body><apply-to xmlns='urn:xmpp:fasten:0' id="msg-id"><retract xmlns='urn:xmpp:message-retract:0'/></apply-to><fallback xmlns='urn:xmpp:fallback:0'/></message>
    "###);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_mark_message_received() -> Result<()> {
    let ConnectedClient {
        connection, client, ..
    } = Client::connected_client().await?;

    let chat = client.get_mod::<mods::Chat>();
    chat.mark_message_received("msg-id".into(), jid!("recv@prose.org"), &MessageType::Chat)?;

    let sent_stanzas = connection.sent_stanza_strings();
    assert_eq!(sent_stanzas.len(), 1);
    assert_snapshot!(sent_stanzas[0], @r###"
        <message xmlns='jabber:client' from="test@prose.org/test" id="id-1" to="recv@prose.org" type="chat"><received xmlns='urn:xmpp:chat-markers:0' id="msg-id"/></message>
    "###);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_set_message_carbons_enabled() -> Result<()> {
    let ConnectedClient {
        connection, client, ..
    } = Client::connected_client().await?;

    let chat = client.get_mod::<mods::Chat>();
    chat.set_message_carbons_enabled(true)?;
    chat.set_message_carbons_enabled(false)?;

    let sent_stanzas = connection.sent_stanza_strings();
    assert_eq!(sent_stanzas.len(), 2);
    assert_snapshot!(sent_stanzas[0], @r###"
        <iq xmlns='jabber:client' id="id-1" type="set"><enable xmlns='urn:xmpp:carbons:2'/></iq>
    "###);
    assert_snapshot!(sent_stanzas[1], @r###"
        <iq xmlns='jabber:client' id="id-2" type="set"><disable xmlns='urn:xmpp:carbons:2'/></iq>
    "###);

    Ok(())
}
