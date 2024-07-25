// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use chrono::Duration;
use minidom::Element;

use pretty_assertions::assert_eq;
use prose_core_client::dtos::{MessageSender, MucId, OccupantId, Reaction, UserId};
use prose_core_client::{muc_id, occupant_id, user_id};
use prose_proc_macros::mt_test;
use prose_xmpp::TimeProvider;

use crate::tests::client::helpers::{ElementExt, JoinRoomStrategy, TestClient};

#[mt_test]
async fn test_reactions() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room_id = muc_id!("room@conf.prose.org");
    let message_id = client.get_next_message_id();

    let messages = vec![
        Element::from_pretty_printed_xml(r#"
          <result id='msg-id-1' queryid='cf8b5ab8-d017-4f86-89dc-0fba750e121c' xmlns='urn:xmpp:mam:2'>
            <forwarded xmlns='urn:xmpp:forward:0'>
              <delay stamp='2024-07-02T07:01:23Z' xmlns='urn:xmpp:delay' />
              <message type='groupchat' xml:lang='en' from='room@conf.prose.org/jonas’' id='id-1' xmlns='jabber:client'>
                <body>I recommend `ncdu -x /`</body>
              </message>
            </forwarded>
          </result>
        "#)?.try_into()?,
        Element::from_pretty_printed_xml(r#"
          <result id='msg-id-2' queryid='cf8b5ab8-d017-4f86-89dc-0fba750e121c' xmlns='urn:xmpp:mam:2'>
            <forwarded xmlns='urn:xmpp:forward:0'>
              <delay stamp='2024-07-02T07:04:10Z' xmlns='urn:xmpp:delay' />
              <message type='groupchat' xml:lang='en' from='room@conf.prose.org/drs' id='id-2' xmlns='jabber:client'>
                <reactions id='msg-id-1' xmlns='urn:xmpp:reactions:0'>
                  <reaction>👍</reaction>
                </reactions>
              </message>
            </forwarded>
          </result>
        "#)?.try_into()?,
        Element::from_pretty_printed_xml(r#"
          <result id='msg-id-3' queryid='cf8b5ab8-d017-4f86-89dc-0fba750e121c' xmlns='urn:xmpp:mam:2'>
            <forwarded xmlns='urn:xmpp:forward:0'>
              <delay stamp='2024-07-02T09:37:32Z' xmlns='urn:xmpp:delay' />
              <message type='groupchat' xml:lang='en' from='room@conf.prose.org/huxx' id='id-3' xmlns='jabber:client'>
                <reactions id='msg-id-1' xmlns='urn:xmpp:reactions:0'>
                  <reaction>👍</reaction>
                </reactions>
              </message>
            </forwarded>
          </result>
        "#)?.try_into()?,
        Element::from_pretty_printed_xml(r#"
          <result id='msg-id-4' queryid='cf8b5ab8-d017-4f86-89dc-0fba750e121c' xmlns='urn:xmpp:mam:2'>
            <forwarded xmlns='urn:xmpp:forward:0'>
              <delay stamp='2024-07-02T09:37:32Z' xmlns='urn:xmpp:delay' />
              <message type='groupchat' xml:lang='en' from='room@conf.prose.org/flux' id='id-4' xmlns='jabber:client'>
                <reactions id='msg-id-1' xmlns='urn:xmpp:reactions:0'>
                  <reaction>👍🏽</reaction>
                </reactions>
              </message>
            </forwarded>
          </result>
        "#)?.try_into()?
    ];

    client
        .join_room_with_strategy(
            room_id.clone(),
            "anon-id",
            JoinRoomStrategy::default().with_catch_up_handler(move |client, room_id| {
                client.expect_muc_catchup_with_config(
                    room_id,
                    client.time_provider.now()
                        - Duration::seconds(client.app_config.max_catchup_duration_secs),
                    messages,
                );
            }),
        )
        .await?;

    let room = client.get_room(room_id).await.to_generic_room();
    let messages = room.load_messages_with_ids(&[message_id]).await?;

    assert_eq!(1, messages.len());

    let message = messages.get(0).unwrap();

    assert_eq!(
        vec![
            Reaction {
                emoji: "👍".into(),
                from: vec![
                    MessageSender {
                        id: occupant_id!("room@conf.prose.org/drs").into(),
                        name: "Drs".to_string(),
                        avatar: None,
                    },
                    MessageSender {
                        id: occupant_id!("room@conf.prose.org/huxx").into(),
                        name: "Huxx".to_string(),
                        avatar: None,
                    }
                ]
            },
            Reaction {
                emoji: "👍🏽".into(),
                from: vec![MessageSender {
                    id: occupant_id!("room@conf.prose.org/flux").into(),
                    name: "Flux".to_string(),
                    avatar: None,
                }]
            }
        ],
        message.reactions
    );

    Ok(())
}
