// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use pretty_assertions::assert_eq;

use prose_core_client::domain::shared::models::AnonOccupantId;
use prose_core_client::dtos::{MucId, SendMessageRequest, SendMessageRequestBody, UserId};
use prose_core_client::{muc_id, user_id, ClientEvent, ClientRoomEventType};
use prose_proc_macros::mt_test;

use crate::{event, recv, room_event, send};

use super::helpers::TestClient;

#[mt_test]
async fn test_joins_room() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    client
        .join_room(muc_id!("room@conference.prose.org"), "anon-id")
        .await?;

    Ok(())
}

#[mt_test]
async fn test_receives_chat_states() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room_id = muc_id!("room@conference.prose.org");
    let occupant_id = client.build_occupant_id(&room_id);

    client.join_room(room_id.clone(), "anon-id").await?;

    let room = client.get_room(room_id.clone()).await.to_generic_room();

    client.push_ctx(
        [
            ("OCCUPANT_ID".into(), occupant_id.to_string()),
            ("OTHER_OCCUPANT_ID".into(), format!("{room_id}/their-nick")),
            ("OTHER_ANON_OCCUPANT_ID".into(), "their-anon-id".to_string()),
            (
                "OTHER_USER_RESOURCE_ID".into(),
                "user2@prose.org/resource".to_string(),
            ),
            ("ROOM_ID".into(), room_id.to_string()),
            ("STANZA_ID".into(), "stanza-id".to_string()),
        ]
        .into(),
    );

    recv!(
        client,
        r#"
        <presence xmlns="jabber:client" from="{{OTHER_OCCUPANT_ID}}">
            <c xmlns="http://jabber.org/protocol/caps" hash="sha-1" node="http://conversations.im" ver="VaFH3zLveT77pOMcOwsKdlw2IPE=" />
            <occupant-id xmlns="urn:xmpp:occupant-id:0" id="{{OTHER_ANON_OCCUPANT_ID}}" />
            <x xmlns="http://jabber.org/protocol/muc#user">
                <item affiliation="member" jid="{{OTHER_USER_RESOURCE_ID}}" role="participant" />
            </x>
        </presence>
        "#
    );

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::ParticipantsChanged
    );

    client.expect_send_vard_request(&user_id!("user2@prose.org"));
    client.receive_not_found_iq_response();

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::ParticipantsChanged
    );

    client.receive_next().await;

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{OTHER_OCCUPANT_ID}}" id="message-id-1" to="{{USER_RESOURCE_ID}}" type="groupchat" xml:lang="en">
            <composing xmlns="http://jabber.org/protocol/chatstates" />
            <occupant-id xmlns="urn:xmpp:occupant-id:0" id="{{OTHER_ANON_OCCUPANT_ID}}" />
        </message>
        "#
    );

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::ComposingUsersChanged
    );

    client.receive_next().await;

    let composing_users = room.load_composing_users().await?;
    assert_eq!(1, composing_users.len());
    assert_eq!(user_id!("user2@prose.org"), composing_users[0].id);

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{OTHER_OCCUPANT_ID}}" id="message-id-2" to="{{USER_RESOURCE_ID}}" type="groupchat" xml:lang="en">
          <body>Hello World</body>
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <occupant-id xmlns="urn:xmpp:occupant-id:0" id="{{OTHER_ANON_OCCUPANT_ID}}" />
          <stanza-id xmlns="urn:xmpp:sid:0" by="{{ROOM_ID}}" id="{{STANZA_ID}}" />
        </message>
        "#
    );

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::ComposingUsersChanged
    );
    event!(client, ClientEvent::SidebarChanged);

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::MessagesAppended {
            message_ids: vec!["message-id-2".into()]
        }
    );

    client.receive_next().await;

    let composing_users = room.load_composing_users().await?;
    assert!(composing_users.is_empty());

    let messages = room
        .load_messages_with_ids(&["message-id-2".into()])
        .await?;
    assert_eq!(1, messages.len());
    assert_eq!(Some("stanza-id".into()), messages[0].stanza_id);

    Ok(())
}

#[mt_test]
async fn test_sends_and_updates_message_to_muc_room() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room_id = muc_id!("room@conference.prose.org");
    let occupant_id = client.build_occupant_id(&room_id);
    let anon_occupant_id = AnonOccupantId::from("anon-occupant-id");

    client
        .join_room(room_id.clone(), anon_occupant_id.clone())
        .await?;

    client.push_ctx(
        [
            ("OCCUPANT_ID".into(), occupant_id.to_string()),
            ("ROOM_ID".into(), room_id.to_string()),
            ("ANON_OCCUPANT_ID".into(), anon_occupant_id.to_string()),
        ]
        .into(),
    );

    let message_id = client.get_next_id();

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="{{ROOM_ID}}" type="groupchat">
          <body>Hello</body>
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <store xmlns="urn:xmpp:hints" />
        </message>
        "#
    );

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::MessagesAppended {
            message_ids: vec![message_id.clone().into()]
        }
    );

    let room = client.get_room(room_id.clone()).await.to_generic_room();
    room.send_message(SendMessageRequest {
        body: Some(SendMessageRequestBody {
            text: "Hello".to_string(),
            mentions: vec![],
        }),
        attachments: vec![],
    })
    .await?;

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{OCCUPANT_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="groupchat" xml:lang="en">
          <body>Hello</body>
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <store xmlns="urn:xmpp:hints" />
          <occupant-id xmlns="urn:xmpp:occupant-id:0" id="{{ANON_OCCUPANT_ID}}" />
          <stanza-id xmlns="urn:xmpp:sid:0" by="{{ROOM_ID}}" id="opZdWmO7r50ee_aGKnWvBMbK" />
        </message>
        "#
    );

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::MessagesUpdated {
            message_ids: vec![message_id.clone().into()]
        }
    );

    client.receive_next().await;

    let messages = room
        .load_messages_with_ids(&[message_id.clone().into()])
        .await?;
    assert_eq!(1, messages.len());
    assert_eq!("Hello", messages[0].body);
    assert_eq!(
        Some("opZdWmO7r50ee_aGKnWvBMbK".into()),
        messages[0].stanza_id,
    );

    client.push_ctx([("INITIAL_MESSAGE_ID".into(), message_id.to_string())].into());

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="{{ROOM_ID}}" type="groupchat">
          <body>Hello World</body>
          <replace xmlns="urn:xmpp:message-correct:0" id="{{INITIAL_MESSAGE_ID}}" />
          <store xmlns="urn:xmpp:hints" />
        </message>
        "#
    );

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::MessagesUpdated {
            message_ids: vec![message_id.clone().into()]
        }
    );

    room.update_message(
        message_id.clone().into(),
        SendMessageRequest {
            body: Some(SendMessageRequestBody {
                text: "Hello World".to_string(),
                mentions: vec![],
            }),
            attachments: vec![],
        },
    )
    .await?;

    let messages = room
        .load_messages_with_ids(&[message_id.clone().into()])
        .await?;
    assert_eq!(1, messages.len());
    assert_eq!("Hello World", messages[0].body);

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{OCCUPANT_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="groupchat" xml:lang="en">
          <body>Hello World</body>
          <replace xmlns="urn:xmpp:message-correct:0" id="{{INITIAL_MESSAGE_ID}}" />
          <store xmlns="urn:xmpp:hints" />
          <occupant-id xmlns="urn:xmpp:occupant-id:0" id="{{ANON_OCCUPANT_ID}}" />
          <stanza-id xmlns="urn:xmpp:sid:0" by="{{ROOM_ID}}" id="907z40xwIIuX4b1YH5jRv1ko" />
        </message>
        "#
    );

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::MessagesUpdated {
            message_ids: vec![message_id.clone().into()]
        }
    );

    client.receive_next().await;

    let messages = room
        .load_messages_with_ids(&[message_id.clone().into()])
        .await?;
    assert_eq!(1, messages.len());
    assert_eq!("Hello World", messages[0].body);

    client.pop_ctx();
    client.pop_ctx();

    Ok(())
}
