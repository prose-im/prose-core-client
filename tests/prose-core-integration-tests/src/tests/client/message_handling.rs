// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::tests::client::helpers::{StartDMStrategy, TestClient};
use crate::tests::store;
use crate::{event, recv, room_event, send};
use anyhow::Result;
use chrono::{DateTime, Duration, TimeZone, Utc};
use itertools::Itertools;
use minidom::Element;
use pretty_assertions::assert_eq;
use prose_core_client::domain::messaging::repos::MessagesRepository;
use prose_core_client::domain::shared::models::AnonOccupantId;
use prose_core_client::dtos::{
    AccountId, MucId, RoomId, SendMessageRequest, SendMessageRequestBody, UserId,
};
use prose_core_client::infra::messaging::CachingMessageRepository;
use prose_core_client::test::MessageBuilder;
use prose_core_client::{account_id, muc_id, user_id, ClientEvent, ClientRoomEventType};
use prose_proc_macros::mt_test;
use prose_xmpp::stanza::Message;
use prose_xmpp::{bare, TimeProvider};
use xmpp_parsers::mam::QueryId;

#[mt_test]
async fn test_receives_message_with_same_id_twice() -> Result<()> {
    let client = TestClient::new().await;
    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let other_user_id = user_id!("other@prose.org");
    client.push_ctx([("OTHER_USER_ID", other_user_id.to_string())]);

    let room = client
        .start_dm(other_user_id.clone())
        .await?
        .to_generic_room();

    let message1_id = client.get_next_message_id_with_offset(1);
    let message2_id = client.get_next_message_id_with_offset(2);

    {
        recv!(
            client,
            r#"
            <message xmlns="jabber:client" from="{{OTHER_USER_ID}}" id="message-id" to="{{USER_RESOURCE_ID}}" type="chat">
              <body>Message 1</body>
            </message>
            "#
        );

        event!(client, ClientEvent::SidebarChanged);
        room_event!(
            client,
            room.jid().clone(),
            ClientRoomEventType::MessagesAppended {
                message_ids: vec![message1_id.clone()]
            }
        );
    }
    client.receive_next().await;

    {
        recv!(
            client,
            r#"
            <message xmlns="jabber:client" from="{{OTHER_USER_ID}}" id="message-id" to="{{USER_RESOURCE_ID}}" type="chat">
              <body>Message 2</body>
            </message>
            "#
        );

        event!(client, ClientEvent::SidebarChanged);
        room_event!(
            client,
            room.jid().clone(),
            ClientRoomEventType::MessagesAppended {
                message_ids: vec![message2_id.clone()]
            }
        );
    }
    client.receive_next().await;

    let messages = room
        .load_messages_with_ids(&[message1_id, message2_id])
        .await?
        .into_iter()
        .map(|msg| msg.body.raw)
        .collect::<Vec<_>>();

    assert_eq!(vec!["Message 1", "Message 2"], messages);

    Ok(())
}

#[mt_test]
async fn test_updates_existing_messages_on_catchup() -> Result<()> {
    let store = store().await.expect("Failed to set up store.");
    let client = TestClient::builder().set_store(store.clone()).build().await;

    let account = account_id!("user@prose.org");
    let user_id = user_id!("other@prose.org");
    let room_id = RoomId::User(user_id.clone());

    let message_repo = CachingMessageRepository::new(store.clone());
    message_repo
        .append(
            &account,
            &room_id,
            &[
                MessageBuilder::new_with_index(1)
                    .set_from(account.to_user_id())
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 25, 10, 00, 00).unwrap())
                    .set_id("custom-msg-id-1")
                    .set_server_id(None)
                    .set_remote_id(Some("remote-id-1".into()))
                    .build_message_like(),
                MessageBuilder::new_with_index(2)
                    .set_from(user_id.clone())
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 25, 10, 00, 10).unwrap())
                    .set_id("custom-msg-id-2")
                    .set_server_id(Some("server-id-2".into()))
                    .set_remote_id(Some("remote-id-2".into()))
                    .build_message_like(),
            ],
        )
        .await?;

    client.expect_login(account.to_user_id(), "secret").await?;

    let join_dm_strategy = StartDMStrategy::default().with_catch_up_handler({
        let account = account.clone();
        move |client, user_id| {
            client.expect_catchup_with_config(
                user_id,
                client.time_provider.now()
                    - Duration::seconds(client.app_config.max_catchup_duration_secs),
                vec![
                    MessageBuilder::new_with_index(1)
                        .set_from(account.to_user_id())
                        .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 10, 00, 00).unwrap())
                        .set_remote_id(Some("remote-id-1".into()))
                        .set_server_id(Some("server-id-1".into()))
                        .build_archived_message("", None),
                    MessageBuilder::new_with_index(2)
                        .set_from(user_id.clone())
                        .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 10, 10, 00).unwrap())
                        .set_remote_id(Some("remote-id-2".into()))
                        .set_server_id(Some("server-id-2".into()))
                        .build_archived_message("", None),
                    MessageBuilder::new_with_index(3)
                        .set_from(user_id.clone())
                        .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 10, 20, 00).unwrap())
                        .set_remote_id(Some("remote-id-3".into()))
                        .set_server_id(Some("server-id-3".into()))
                        .build_archived_message("", None),
                ],
            )
        }
    });

    let new_message_id = client.get_next_message_id();

    client
        .start_dm_with_strategy(user_id.clone(), join_dm_strategy)
        .await?;

    let messages = message_repo
        .get_messages_after(&account, &room_id, DateTime::<Utc>::MIN_UTC)
        .await?
        .into_iter()
        .map(|msg| (msg.id, msg.server_id, msg.timestamp))
        .sorted_by(|l, r| l.2.cmp(&r.2))
        .collect::<Vec<_>>();

    assert_eq!(
        vec![
            (
                "custom-msg-id-1".into(),
                Some("server-id-1".into()),
                Utc.with_ymd_and_hms(2024, 04, 26, 10, 00, 00).unwrap()
            ),
            (
                "custom-msg-id-2".into(),
                Some("server-id-2".into()),
                Utc.with_ymd_and_hms(2024, 04, 26, 10, 10, 00).unwrap()
            ),
            (
                new_message_id,
                Some("server-id-3".into()),
                Utc.with_ymd_and_hms(2024, 04, 26, 10, 20, 00).unwrap()
            ),
        ],
        messages
    );

    Ok(())
}

#[mt_test]
async fn test_updates_existing_messages_when_loading_from_mam() -> Result<()> {
    let store = store().await.expect("Failed to set up store.");
    let client = TestClient::builder().set_store(store.clone()).build().await;

    let account = account_id!("user@prose.org");
    let user_id = user_id!("other@prose.org");
    let room_id = RoomId::User(user_id.clone());

    client.push_ctx([("OTHER_USER_ID", user_id.to_string())]);

    let message_repo = CachingMessageRepository::new(store.clone());
    message_repo
        .append(
            &account,
            &room_id,
            &[
                // A message by us without a StanzaId
                MessageBuilder::new_with_index(1)
                    .set_from(account.to_user_id())
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 25, 10, 00, 00).unwrap())
                    .set_id("custom-msg-id-1")
                    .set_server_id(None)
                    .set_remote_id(Some("remote-id-1".into()))
                    .build_message_like(),
                // A message by them with a StanzaId
                MessageBuilder::new_with_index(2)
                    .set_from(user_id.clone())
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 25, 10, 00, 10).unwrap())
                    .set_id("custom-msg-id-2")
                    .set_server_id(Some("server-id-2".into()))
                    .set_remote_id(Some("remote-id-2".into()))
                    .build_message_like(),
            ],
        )
        .await?;

    client.expect_login(account.to_user_id(), "secret").await?;

    let room = client.start_dm(user_id.clone()).await?.to_generic_room();

    send!(
        client,
        r#"
        <iq xmlns="jabber:client" id="{{ID:2}}" type="set">
          <query xmlns="urn:xmpp:mam:2" queryid="{{ID:1}}">
            <x xmlns="jabber:x:data" type="submit">
              <field type="hidden" var="FORM_TYPE">
                <value>urn:xmpp:mam:2</value>
              </field>
              <field var="with">
                <value>{{OTHER_USER_ID}}</value>
              </field>
            </x>
            <set xmlns="http://jabber.org/protocol/rsm">
              <max>100</max>
              <before />
            </set>
          </query>
        </iq>
    "#
    );

    let query_id = QueryId(client.id_provider.id_with_offset(1));

    let received_messages = vec![
        // This should update our message
        MessageBuilder::new_with_index(1)
            .set_from(account.to_user_id())
            .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 10, 00, 00).unwrap())
            .set_remote_id(Some("remote-id-1".into()))
            .set_server_id(Some("server-id-1".into()))
            .build_archived_message("", None),
        // This should update their message
        MessageBuilder::new_with_index(2)
            .set_from(user_id.clone())
            .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 10, 10, 00).unwrap())
            .set_remote_id(Some("remote-id-2".into()))
            .set_server_id(Some("server-id-2".into()))
            .build_archived_message("", None),
        // This should create a new message
        MessageBuilder::new_with_index(3)
            .set_from(user_id.clone())
            .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 10, 20, 00).unwrap())
            .set_remote_id(Some("remote-id-3".into()))
            .set_server_id(Some("server-id-3".into()))
            .build_archived_message("", None),
    ];

    for mut archived_message in received_messages.into_iter() {
        archived_message.query_id = Some(query_id.clone());

        let message = Message::new().set_archived_message(archived_message);
        client.receive_element(Element::from(message), file!(), line!());
    }

    recv!(
        client,
        r#"
            <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="result">
                <fin xmlns="urn:xmpp:mam:2" complete="true">
                    <set xmlns="http://jabber.org/protocol/rsm" />
                </fin>
            </iq>
            "#
    );

    let new_message_id = client.get_next_message_id();

    _ = room.load_latest_messages().await?;

    let messages = message_repo
        .get_messages_after(&account, &room_id, DateTime::<Utc>::MIN_UTC)
        .await?
        .into_iter()
        .map(|msg| (msg.id, msg.server_id, msg.timestamp))
        .sorted_by(|l, r| l.2.cmp(&r.2))
        .collect::<Vec<_>>();

    assert_eq!(
        vec![
            (
                "custom-msg-id-1".into(),
                Some("server-id-1".into()),
                Utc.with_ymd_and_hms(2024, 04, 26, 10, 00, 00).unwrap()
            ),
            (
                "custom-msg-id-2".into(),
                Some("server-id-2".into()),
                Utc.with_ymd_and_hms(2024, 04, 26, 10, 10, 00).unwrap()
            ),
            (
                new_message_id,
                Some("server-id-3".into()),
                Utc.with_ymd_and_hms(2024, 04, 26, 10, 20, 00).unwrap()
            ),
        ],
        messages
    );

    Ok(())
}

#[mt_test]
async fn test_sends_and_updates_message_to_muc_room() -> Result<()> {
    let store = store().await.expect("Failed to set up store.");
    let client = TestClient::builder().set_store(store.clone()).build().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room_id = muc_id!("room@conference.prose.org");
    let occupant_id = client.build_occupant_id(&room_id);
    let anon_occupant_id = AnonOccupantId::from("anon-occupant-id");

    client
        .join_room(room_id.clone(), anon_occupant_id.clone())
        .await?;

    client.push_ctx([
        ("OCCUPANT_ID", occupant_id.to_string()),
        ("ROOM_ID", room_id.to_string()),
        ("ANON_OCCUPANT_ID", anon_occupant_id.to_string()),
    ]);

    let message_id = client.get_next_message_id();

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{MSG_ID}}" to="{{ROOM_ID}}" type="groupchat">
          <body>Hello</body>
          <content xmlns="urn:xmpp:content" type="text/markdown">Hello</content>
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
            text: "Hello".into(),
        }),
        attachments: vec![],
    })
    .await?;

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{OCCUPANT_ID}}" id="{{LAST_MSG_ID}}" to="{{USER_RESOURCE_ID}}" type="groupchat" xml:lang="en">
          <body>Hello</body>
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <store xmlns="urn:xmpp:hints" />
          <occupant-id xmlns="urn:xmpp:occupant-id:0" id="{{ANON_OCCUPANT_ID}}" />
          <stanza-id xmlns="urn:xmpp:sid:0" by="{{ROOM_ID}}" id="opZdWmO7r50ee_aGKnWvBMbK" />
        </message>
        "#
    );

    client.receive_next().await;

    let messages = CachingMessageRepository::new(store)
        .get(
            &bare!("user@prose.org").into(),
            &room_id.clone().into(),
            &message_id,
        )
        .await?;

    assert_eq!(1, messages.len());
    assert_eq!(
        Some("opZdWmO7r50ee_aGKnWvBMbK".into()),
        messages[0].server_id,
    );

    client.push_ctx([("INITIAL_MESSAGE_ID", message_id.to_string())]);

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{MSG_ID}}" to="{{ROOM_ID}}" type="groupchat">
          <body>Hello World</body>
          <content xmlns="urn:xmpp:content" type="text/markdown">Hello World</content>
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
                text: "Hello World".into(),
            }),
            attachments: vec![],
        },
    )
    .await?;

    let messages = room
        .load_messages_with_ids(&[message_id.clone().into()])
        .await?;
    assert_eq!(1, messages.len());
    assert_eq!("<p>Hello World</p>", messages[0].body.html.as_ref());

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{OCCUPANT_ID}}" id="{{LAST_MSG_ID}}" to="{{USER_RESOURCE_ID}}" type="groupchat" xml:lang="en">
          <body>Hello World</body>
          <replace xmlns="urn:xmpp:message-correct:0" id="{{INITIAL_MESSAGE_ID}}" />
          <store xmlns="urn:xmpp:hints" />
          <occupant-id xmlns="urn:xmpp:occupant-id:0" id="{{ANON_OCCUPANT_ID}}" />
          <stanza-id xmlns="urn:xmpp:sid:0" by="{{ROOM_ID}}" id="907z40xwIIuX4b1YH5jRv1ko" />
        </message>
        "#
    );

    client.receive_next().await;

    let messages = room
        .load_messages_with_ids(&[message_id.clone().into()])
        .await?;
    assert_eq!(1, messages.len());
    assert_eq!("<p>Hello World</p>", messages[0].body.html.as_ref());

    client.pop_ctx();
    client.pop_ctx();

    Ok(())
}
