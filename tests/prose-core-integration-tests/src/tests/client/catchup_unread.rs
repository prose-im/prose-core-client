// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use chrono::{Duration, TimeZone, Utc};
use minidom::Element;
use pretty_assertions::assert_eq;
use xmpp_parsers::mam::QueryId;

use prose_core_client::domain::messaging::models::{MessageLikePayload, MessageRef};
use prose_core_client::domain::messaging::repos::MessagesRepository;
use prose_core_client::domain::settings::models::SyncedRoomSettings;
use prose_core_client::domain::shared::models::AccountId;
use prose_core_client::dtos::{Mention, MucId, OccupantId, RoomId, UnicodeScalarIndex, UserId};
use prose_core_client::infra::messaging::CachingMessageRepository;
use prose_core_client::test::{ConstantTimeProvider, MessageBuilder};
use prose_core_client::{account_id, muc_id, occupant_id, user_id, ClientEvent};
use prose_proc_macros::mt_test;
use prose_xmpp::stanza::Message;
use prose_xmpp::TimeProvider;

use crate::tests::client::helpers::{JoinRoomStrategy, StartDMStrategy, TestClient};
use crate::tests::store;
use crate::{event, recv, send};

#[mt_test]
async fn test_maintains_message_count_from_prior_runs() -> Result<()> {
    let store = store().await.expect("Failed to set up store.");

    let account = account_id!("user@prose.org");
    let muc_id = muc_id!("room@conf.prose.org");
    let room_id = RoomId::Muc(muc_id.clone());

    let message_repo = CachingMessageRepository::new(store.clone());
    message_repo
        .append(
            &account,
            &room_id,
            &[
                MessageBuilder::new_with_index(1)
                    .set_from(occupant_id!("room@conf.prose.org/friend"))
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 25, 10, 00, 00).unwrap())
                    .build_message_like(),
                MessageBuilder::new_with_index(2)
                    .set_from(occupant_id!("room@conf.prose.org/friend"))
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 10, 00, 00).unwrap())
                    .build_message_like(),
                MessageBuilder::new_with_index(3)
                    .set_from(occupant_id!("room@conf.prose.org/friend"))
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 11, 00, 00).unwrap())
                    .build_message_like(),
            ],
        )
        .await?;

    let now = Utc::now();

    let client = TestClient::builder()
        .set_store(store)
        .set_time_provider(ConstantTimeProvider::new(now.clone()))
        .build()
        .await;
    client.expect_login(account.to_user_id(), "secret").await?;

    let mut join_room_strategy = JoinRoomStrategy::default();
    join_room_strategy.room_settings = Some(SyncedRoomSettings {
        room_id: room_id.clone(),
        encryption_enabled: false,
        last_read_message: Some(MessageRef {
            id: MessageBuilder::id_for_index(1),
            timestamp: Utc.with_ymd_and_hms(2024, 04, 25, 10, 00, 00).unwrap(),
        }),
    });
    join_room_strategy.expect_catchup = Box::new(|client, room_id| {
        client.expect_muc_catchup_with_config(
            room_id,
            client.time_provider.now()
                - Duration::seconds(client.app_config.max_catchup_duration_secs),
            [
                MessageBuilder::new_with_index(4)
                    .set_from(occupant_id!("room@conf.prose.org/friend"))
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 12, 00, 00).unwrap())
                    .build_archived_message("", None),
                MessageBuilder::new_with_index(5)
                    .set_from(occupant_id!("room@conf.prose.org/friend"))
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 13, 00, 00).unwrap())
                    .build_archived_message("", None),
            ],
        )
    });

    client
        .join_room_with_strategy(muc_id.clone(), "anon-id", join_room_strategy)
        .await?;

    let sidebar_items = client.sidebar.sidebar_items().await;
    assert_eq!(1, sidebar_items.len());

    let sidebar_item = sidebar_items
        .get(0)
        .expect("Expected at least one SidebarItem");
    assert_eq!(4, sidebar_item.unread_count);

    Ok(())
}

#[mt_test]
async fn test_loads_unread_messages() -> Result<()> {
    let store = store().await.expect("Failed to set up store.");

    let account = account_id!("user@prose.org");
    let muc_id = muc_id!("room@conf.prose.org");
    let room_id = RoomId::Muc(muc_id.clone());

    let message_repo = CachingMessageRepository::new(store.clone());
    message_repo
        .append(
            &account,
            &room_id,
            &[
                MessageBuilder::new_with_index(1)
                    .set_from(occupant_id!("room@conf.prose.org/friend"))
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 25, 10, 00, 00).unwrap())
                    .build_message_like(),
                MessageBuilder::new_with_index(2)
                    .set_from(occupant_id!("room@conf.prose.org/friend"))
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 10, 00, 00).unwrap())
                    .build_message_like(),
                MessageBuilder::new_with_index(3)
                    .set_from(occupant_id!("room@conf.prose.org/friend"))
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 11, 00, 00).unwrap())
                    .build_message_like(),
            ],
        )
        .await?;

    let now = Utc::now();

    let client = TestClient::builder()
        .set_store(store)
        .set_time_provider(ConstantTimeProvider::new(now.clone()))
        .build()
        .await;
    client.expect_login(account.to_user_id(), "secret").await?;

    let mut join_room_strategy = JoinRoomStrategy::default();
    join_room_strategy.room_settings = Some(SyncedRoomSettings {
        room_id: room_id.clone(),
        encryption_enabled: false,
        last_read_message: Some(MessageRef {
            id: MessageBuilder::id_for_index(1),
            timestamp: Utc.with_ymd_and_hms(2024, 04, 25, 10, 00, 00).unwrap(),
        }),
    });

    client
        .join_room_with_strategy(muc_id.clone(), "anon-id", join_room_strategy)
        .await?;

    let room = client.get_room(room_id).await.to_generic_room();
    let unread_messages = room.load_unread_messages().await?;

    assert_eq!(
        vec![
            MessageBuilder::stanza_id_for_index(2),
            MessageBuilder::stanza_id_for_index(3),
        ],
        unread_messages
            .into_iter()
            .filter_map(|message| message.stanza_id)
            .collect::<Vec<_>>()
    );

    Ok(())
}

#[mt_test]
async fn test_updates_unread_count_after_sync() -> Result<()> {
    let store = store().await.expect("Failed to set up store.");

    let account = account_id!("user@prose.org");
    let user_id = user_id!("friend@prose.org");
    let room_id = RoomId::User(user_id.clone());

    let message_repo = CachingMessageRepository::new(store.clone());
    message_repo
        .append(
            &account,
            &room_id,
            &[
                MessageBuilder::new_with_index(1)
                    .set_from(user_id.clone())
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 25, 10, 00, 00).unwrap())
                    .set_payload(MessageLikePayload::Message {
                        body: "Hello @ou".to_string(),
                        attachments: vec![],
                        mentions: vec![Mention {
                            user: user_id!("user@prose.org"),
                            range: UnicodeScalarIndex::new(6)..UnicodeScalarIndex::new(9),
                        }],
                        encryption_info: None,
                        is_transient: false,
                    })
                    .build_message_like(),
                MessageBuilder::new_with_index(2)
                    .set_from(user_id.clone())
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 10, 00, 00).unwrap())
                    .build_message_like(),
                MessageBuilder::new_with_index(3)
                    .set_from(user_id.clone())
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 11, 00, 00).unwrap())
                    .set_payload(MessageLikePayload::Message {
                        body: "Hello @ou".to_string(),
                        attachments: vec![],
                        mentions: vec![Mention {
                            user: user_id!("user@prose.org"),
                            range: UnicodeScalarIndex::new(6)..UnicodeScalarIndex::new(9),
                        }],
                        encryption_info: None,
                        is_transient: false,
                    })
                    .build_message_like(),
            ],
        )
        .await?;

    let client = TestClient::builder().set_store(store).build().await;
    client.expect_login(account.to_user_id(), "secret").await?;

    client.start_dm(user_id.clone()).await?;

    let sidebar_items = client.sidebar.sidebar_items().await;
    assert_eq!(1, sidebar_items.len());

    let sidebar_item = sidebar_items
        .get(0)
        .expect("Expected at least one SidebarItem");
    assert_eq!(3, sidebar_item.unread_count);
    assert_eq!(2, sidebar_item.mentions_count);

    client.push_ctx(
        [
            ("OTHER_USER_ID".into(), user_id.to_string()),
            ("MSG_ID".into(), MessageBuilder::id_for_index(2).to_string()),
            (
                "MSG_TIMESTAMP".into(),
                Utc.with_ymd_and_hms(2024, 04, 26, 10, 00, 00)
                    .unwrap()
                    .to_rfc3339(),
            ),
        ]
        .into(),
    );
    recv!(
        client,
        r#"
      <message xmlns="jabber:client" from="{{USER_ID}}" id="X5HyLLwZYrGEJGODIb0ek4FM" to="{{USER_RESOURCE_ID}}" type="headline">
        <event xmlns="http://jabber.org/protocol/pubsub#event">
          <items node="https://prose.org/protocol/room_settings">
            <item id="{{OTHER_USER_ID}}" publisher="{{USER_ID}}">
              <room-settings xmlns="https://prose.org/protocol/room_settings" room-id="user:{{OTHER_USER_ID}}">
                <message-ref xmlns="https://prose.org/protocol/message_ref" id="{{MSG_ID}}" ts="{{MSG_TIMESTAMP}}" />
              </room-settings>
            </item>
          </items>
        </event>
      </message>
      "#
    );
    client.pop_ctx();

    event!(client, ClientEvent::SidebarChanged);

    client.receive_next().await;

    let sidebar_items = client.sidebar.sidebar_items().await;
    assert_eq!(1, sidebar_items.len());

    let sidebar_item = sidebar_items
        .get(0)
        .expect("Expected at least one SidebarItem");
    assert_eq!(1, sidebar_item.unread_count);
    assert_eq!(1, sidebar_item.mentions_count);

    Ok(())
}

#[mt_test]
async fn test_marks_first_unread_message() -> Result<()> {
    let store = store().await.expect("Failed to set up store.");

    let account = user_id!("user@prose.org");
    let user_id = user_id!("friend@prose.org");
    let room_id = RoomId::User(user_id.clone());

    let client = TestClient::builder().set_store(store).build().await;
    client.expect_login(account, "secret").await?;

    let mut strategy = StartDMStrategy::default();
    strategy.room_settings = Some(SyncedRoomSettings {
        room_id: room_id.clone(),
        encryption_enabled: false,
        last_read_message: Some(MessageRef {
            id: MessageBuilder::id_for_index(2),
            timestamp: Utc.with_ymd_and_hms(2024, 04, 26, 11, 00, 00).unwrap(),
        }),
    });

    let room = client
        .start_dm_with_strategy(user_id.clone(), strategy)
        .await?
        .to_generic_room();

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
                <value>friend@prose.org</value>
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
        MessageBuilder::new_with_index(1)
            .set_from(user_id.clone())
            .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 10, 00, 00).unwrap())
            .build_archived_message("", None),
        MessageBuilder::new_with_index(2)
            .set_from(user_id.clone())
            .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 11, 00, 00).unwrap())
            .build_archived_message("", None),
        MessageBuilder::new_with_index(3)
            .set_from(user_id.clone())
            .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 12, 00, 00).unwrap())
            .build_archived_message("", None),
        MessageBuilder::new_with_index(4)
            .set_from(user_id.clone())
            .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 13, 00, 00).unwrap())
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

    let messages = room.load_latest_messages().await?;

    assert_eq!(4, messages.messages.len());
    assert_eq!(false, messages.messages[0].is_last_read);
    assert_eq!(true, messages.messages[1].is_last_read);
    assert_eq!(false, messages.messages[2].is_last_read);
    assert_eq!(false, messages.messages[3].is_last_read);

    Ok(())
}

#[mt_test]
async fn test_mark_as_unread_saves_settings() -> Result<()> {
    let store = store().await.expect("Failed to set up store.");

    let account = account_id!("user@prose.org");
    let muc_id = muc_id!("room@conf.prose.org");
    let room_id = RoomId::Muc(muc_id.clone());

    let message_repo = CachingMessageRepository::new(store.clone());
    message_repo
        .append(
            &account,
            &room_id,
            &[
                MessageBuilder::new_with_index(1)
                    .set_from(occupant_id!("room@conf.prose.org/friend"))
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 25, 10, 00, 00).unwrap())
                    .build_message_like(),
                MessageBuilder::new_with_index(2)
                    .set_from(occupant_id!("room@conf.prose.org/friend"))
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 10, 00, 00).unwrap())
                    .build_message_like(),
                MessageBuilder::new_with_index(3)
                    .set_from(occupant_id!("room@conf.prose.org/friend"))
                    .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 26, 11, 00, 00).unwrap())
                    .build_message_like(),
            ],
        )
        .await?;

    let now = Utc::now();

    let client = TestClient::builder()
        .set_store(store)
        .set_time_provider(ConstantTimeProvider::new(now.clone()))
        .build()
        .await;
    client.expect_login(account.to_user_id(), "secret").await?;

    let mut join_room_strategy = JoinRoomStrategy::default();
    join_room_strategy.room_settings = Some(SyncedRoomSettings {
        room_id: room_id.clone(),
        encryption_enabled: false,
        last_read_message: Some(MessageRef {
            id: MessageBuilder::id_for_index(1),
            timestamp: Utc.with_ymd_and_hms(2024, 04, 25, 10, 00, 00).unwrap(),
        }),
    });

    client
        .join_room_with_strategy(muc_id.clone(), "anon-id", join_room_strategy)
        .await?;

    let sidebar_items = client.sidebar.sidebar_items().await;
    assert_eq!(1, sidebar_items.len());

    let sidebar_item = sidebar_items
        .get(0)
        .expect("Expected at least one SidebarItem");
    assert_eq!(2, sidebar_item.unread_count);

    client.push_ctx(
        [
            ("ROOM_ID".into(), room_id.to_string()),
            (
                "MESSAGE_ID".into(),
                MessageBuilder::id_for_index(3).to_string(),
            ),
        ]
        .into(),
    );

    send!(
        client,
        r#"
        <iq xmlns="jabber:client" id="{{ID}}" type="set">
          <pubsub xmlns="http://jabber.org/protocol/pubsub">
            <publish node="https://prose.org/protocol/room_settings">
              <item id="{{ROOM_ID}}">
                <room-settings xmlns="https://prose.org/protocol/room_settings" room-id="muc:{{ROOM_ID}}">
                  <message-ref xmlns="https://prose.org/protocol/message_ref" id="{{MESSAGE_ID}}" ts="2024-04-26T11:00:00+00:00" />
                  <encryption type="none" />
                </room-settings>
              </item>
            </publish>
            <publish-options>
              <x xmlns="jabber:x:data" type="submit">
                <field type="hidden" var="FORM_TYPE">
                  <value>http://jabber.org/protocol/pubsub#publish-options</value>
                </field>
                <field type="boolean" var="pubsub#persist_items">
                  <value>true</value>
                </field>
                <field var="pubsub#access_model">
                  <value>whitelist</value>
                </field>
                <field var="pubsub#max_items">
                  <value>256</value>
                </field>
                <field type="list-single" var="pubsub#send_last_published_item">
                  <value>never</value>
                </field>
              </x>
            </publish-options>
          </pubsub>
        </iq>
        "#
    );

    recv!(
        client,
        r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_ID}}" type="result">
          <pubsub xmlns="http://jabber.org/protocol/pubsub">
            <publish node="https://prose.org/protocol/room_settings">
              <item id="{{ROOM_ID}}" />
            </publish>
          </pubsub>
        </iq>
        "#
    );

    client.pop_ctx();

    event!(client, ClientEvent::SidebarChanged);

    let room = client.get_room(room_id).await.to_generic_room();
    room.mark_as_read().await?;

    // This shouldn't do anything, since the last read message did not change.
    room.mark_as_read().await?;

    Ok(())
}
