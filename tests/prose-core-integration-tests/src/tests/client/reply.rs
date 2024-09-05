// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::tests::client::helpers::{StartDMStrategy, TestClient};
use crate::{event, recv, room_event, send};
use anyhow::Result;
use chrono::Duration;
use prose_core_client::dtos::*;
use prose_core_client::test::MessageBuilder;
use prose_core_client::{user_id, ClientEvent, ClientRoomEventType};
use prose_proc_macros::mt_test;
use prose_xmpp::TimeProvider;

#[mt_test]
async fn test_resolves_replies() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let mam_msg1 = MessageBuilder::new_with_index(1)
        .set_payload("Message 1")
        .set_from(user_id!("user@prose.org"))
        .build_archived_message("", None);

    let mut mam_msg2 = MessageBuilder::new_with_index(2)
        .set_payload("Corrected Message 1")
        .set_from(user_id!("user@prose.org"))
        .build_archived_message("", None);
    mam_msg2.forwarded.stanza =
        Some(Box::new(mam_msg2.forwarded.stanza.unwrap().set_replace(
            MessageBuilder::remote_id_for_index(1).into_inner().into(),
        )));

    let mam_msg3 = MessageBuilder::new_with_index(3)
        .set_payload("Message 2")
        .set_from(user_id!("user@prose.org"))
        .build_archived_message("", None);

    let room = client
        .start_dm_with_strategy(
            user_id!("them@prose.org"),
            StartDMStrategy::default().with_catch_up_handler(|client, user_id| {
                client.expect_catchup_with_config(
                    user_id,
                    client.time_provider.now()
                        - Duration::seconds(client.app_config.max_catchup_duration_secs),
                    vec![mam_msg1, mam_msg2, mam_msg3],
                );
            }),
        )
        .await?
        .to_generic_room();

    client.push_ctx([
        (
            "MSG_1_ID",
            MessageBuilder::remote_id_for_index(1).into_inner(),
        ),
        (
            "MSG_2_ID",
            MessageBuilder::remote_id_for_index(3).into_inner(),
        ),
    ]);

    let reply1_id = client.get_next_message_id_with_offset(4);
    let reply2_id = client.get_next_message_id_with_offset(5);

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="them@prose.org" id="message-id" to="{{USER_RESOURCE_ID}}" type="chat">
            <body>Reply 1</body>
            <reply xmlns="urn:xmpp:reply:0" id="{{MSG_1_ID}}" to="{{USER_ID}}" />
            <fallback xmlns="urn:xmpp:fallback:0" for="urn:xmpp:reply:0">
                <body start="0" end="8" />
            </fallback>
        </message>
        "#
    );

    event!(client, ClientEvent::SidebarChanged);
    room_event!(
        client,
        room.jid().clone(),
        ClientRoomEventType::MessagesAppended {
            message_ids: vec![reply1_id.clone()]
        }
    );

    client.receive_next().await;

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="them@prose.org" id="message-id" to="{{USER_RESOURCE_ID}}" type="chat">
            <body>&gt; Message 2
Reply 2</body>
            <reply xmlns="urn:xmpp:reply:0" id="{{MSG_2_ID}}" to="{{USER_ID}}" />
            <fallback xmlns="urn:xmpp:fallback:0" for="urn:xmpp:reply:0">
                <body start="0" end="12" />
            </fallback>
            </message>
        "#
    );

    event!(client, ClientEvent::SidebarChanged);
    room_event!(
        client,
        room.jid().clone(),
        ClientRoomEventType::MessagesAppended {
            message_ids: vec![reply2_id.clone()]
        }
    );

    client.receive_next().await;

    let messages = room.load_messages_with_ids(&[reply1_id, reply2_id]).await?;

    assert_eq!(2, messages.len());
    assert_eq!("Reply 1", messages[0].body.raw);
    assert_eq!(
        "Corrected Message 1",
        messages[0].reply_to.clone().unwrap().body.unwrap()
    );
    assert_eq!("Reply 2", messages[1].body.raw);
    assert_eq!(
        "Message 2",
        messages[1].reply_to.clone().unwrap().body.unwrap()
    );

    Ok(())
}
