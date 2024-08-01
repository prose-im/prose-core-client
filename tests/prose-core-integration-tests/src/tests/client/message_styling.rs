// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::dtos::*;
use prose_core_client::{user_id, ClientEvent, ClientRoomEventType};
use prose_proc_macros::mt_test;

use crate::tests::client::helpers::TestClient;
use crate::{event, recv, room_event, send};

#[mt_test]
async fn test_send_markdown_message() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room = client
        .start_dm(user_id!("them@prose.org"))
        .await?
        .to_generic_room();

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="them@prose.org" type="chat">
          <body>Some *bold*, _italic_, ~strikethrough~ and *_bold italic_* text.</body>
          <content xmlns="urn:xmpp:content" type="text/markdown">Some **bold**, _italic_, ~~strikethrough~~ and **_bold italic_** text.</content>
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <store xmlns="urn:xmpp:hints" />
        </message>
        "#
    );

    room_event!(
        client,
        room.jid().clone(),
        ClientRoomEventType::MessagesAppended {
            message_ids: vec![client.get_last_id().into()]
        }
    );

    room.send_message(SendMessageRequest {
        body: Some(SendMessageRequestBody {
            text: "Some **bold**, _italic_, ~~strikethrough~~ and **_bold italic_** text.".into(),
        }),
        attachments: vec![],
    })
    .await?;

    Ok(())
}

#[mt_test]
async fn test_receive_markdown_message() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room = client
        .start_dm(user_id!("them@prose.org"))
        .await?
        .to_generic_room();

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="them@prose.org/res" id="my-message-id" to="{{USER_RESOURCE_ID}}" type="chat">
          <body>Some *bold*, _italic_, ~strikethrough~ and *_bold italic_* text.</body>
          <content xmlns="urn:xmpp:content" type="text/markdown">Some **bold**, _italic_, ~~strikethrough~~ and **_bold italic_** text.</content>
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <store xmlns="urn:xmpp:hints" />
        </message>
        "#
    );

    event!(client, ClientEvent::SidebarChanged);
    room_event!(
        client,
        room.jid().clone(),
        ClientRoomEventType::MessagesAppended {
            message_ids: vec!["my-message-id".into()]
        }
    );

    client.receive_next().await;

    let messages = room
        .load_messages_with_ids(&["my-message-id".into()])
        .await?;

    let message = messages.first().unwrap();

    assert_eq!(
        &message.body.raw,
        "Some **bold**, _italic_, ~~strikethrough~~ and **_bold italic_** text."
    );
    assert_eq!(
        message.body.html.as_ref(),
        "<p>Some <strong>bold</strong>, <em>italic</em>, <del>strikethrough</del> and <strong><em>bold italic</em></strong> text.</p>"
    );

    Ok(())
}

#[mt_test]
async fn test_receive_basic_message() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room = client
        .start_dm(user_id!("them@prose.org"))
        .await?
        .to_generic_room();

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="them@prose.org/res" id="my-message-id" to="{{USER_RESOURCE_ID}}" type="chat">
          <body>Some *bold* text.
And a new line
And another newline</body>
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <store xmlns="urn:xmpp:hints" />
        </message>
        "#
    );

    event!(client, ClientEvent::SidebarChanged);
    room_event!(
        client,
        room.jid().clone(),
        ClientRoomEventType::MessagesAppended {
            message_ids: vec!["my-message-id".into()]
        }
    );

    client.receive_next().await;

    let messages = room
        .load_messages_with_ids(&["my-message-id".into()])
        .await?;

    let message = messages.first().unwrap();

    assert_eq!(
        "Some *bold* text.\nAnd a new line\nAnd another newline",
        &message.body.raw,
    );
    assert_eq!(
        "<p>Some *bold* text.<br/>And a new line<br/>And another newline</p>",
        message.body.html.as_ref(),
    );

    Ok(())
}
