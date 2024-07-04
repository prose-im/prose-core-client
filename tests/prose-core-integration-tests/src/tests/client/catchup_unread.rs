// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use chrono::{Duration, NaiveDate, TimeZone, Utc};
use minidom::Element;
use pretty_assertions::assert_eq;
use xmpp_parsers::mam::QueryId;

use prose_core_client::domain::messaging::models::{ArchivedMessageRef, MessageLikePayload};
use prose_core_client::domain::messaging::repos::MessagesRepository;
use prose_core_client::domain::rooms::models::RoomSidebarState;
use prose_core_client::domain::settings::models::SyncedRoomSettings;
use prose_core_client::domain::shared::models::AccountId;
use prose_core_client::domain::sidebar::models::BookmarkType;
use prose_core_client::dtos::{
    Bookmark, Mention, MucId, OccupantId, RoomId, UnicodeScalarIndex, UserId,
};
use prose_core_client::infra::messaging::CachingMessageRepository;
use prose_core_client::test::{ConstantTimeProvider, MessageBuilder};
use prose_core_client::{
    account_id, muc_id, occupant_id, user_id, ClientEvent, ClientRoomEventType,
};
use prose_proc_macros::mt_test;
use prose_xmpp::stanza::Message;
use prose_xmpp::TimeProvider;

use crate::tests::client::helpers::{JoinRoomStrategy, LoginStrategy, StartDMStrategy, TestClient};
use crate::tests::store;
use crate::{event, recv, room_event, send};

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
        last_read_message: Some(ArchivedMessageRef {
            stanza_id: MessageBuilder::stanza_id_for_index(1),
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
async fn test_rounds_timestamps() -> Result<()> {
    let store = store().await.expect("Failed to set up store.");

    let account = account_id!("user@prose.org");
    let user_id = user_id!("other@prose.org");
    let time_provider = ConstantTimeProvider::default();

    let client = TestClient::builder()
        .set_store(store)
        .set_time_provider(time_provider.clone())
        .build()
        .await;
    client.expect_login(account.to_user_id(), "secret").await?;

    client.push_ctx([("OTHER_USER_ID", user_id.to_string())]);

    // We receive a message at 2024-04-05 10:00:00.550…
    {
        time_provider.set_ymd_hms_millis(2024, 04, 05, 10, 00, 00, 550);

        // Just double-check once that time_provider is working…
        assert_eq!(
            Utc.from_utc_datetime(
                &NaiveDate::from_ymd_opt(2024, 04, 05)
                    .unwrap()
                    .and_hms_milli_opt(10, 00, 00, 550)
                    .unwrap(),
            ),
            time_provider.now()
        );

        recv!(
            client,
            r#"
            <message xmlns="jabber:client" from="{{OTHER_USER_ID}}/res" id="id-1" type="chat" xml:lang="en">
              <body>Hello</body>
              <stanza-id xmlns="urn:xmpp:sid:0" by="{{USER_ID}}" id="stanza-id-1" />
            </message>
            "#
        );

        client.expect_load_synced_room_settings(user_id.clone(), None);

        client.expect_catchup(&user_id);
        client.expect_set_bookmark(user_id.clone(), "Other", BookmarkType::DirectMessage);

        event!(client, ClientEvent::SidebarChanged);
    }
    client.receive_next().await;

    // Check that we have one SidebarItem with an unread_count of 1
    let sidebar_items = client.sidebar.sidebar_items().await;
    assert_eq!(1, sidebar_items.len());
    assert_eq!(1, sidebar_items[0].unread_count);

    // We receive another message at 2024-04-05 10:00:00.750…
    {
        time_provider.set_ymd_hms_millis(2024, 04, 05, 10, 00, 00, 750);

        recv!(
            client,
            r#"
            <message xmlns="jabber:client" from="{{OTHER_USER_ID}}/res" id="id-2" type="chat" xml:lang="en">
              <body>Hello</body>
              <stanza-id xmlns="urn:xmpp:sid:0" by="{{USER_ID}}" id="stanza-id-2" />
            </message>
            "#
        );

        event!(client, ClientEvent::SidebarChanged);
        room_event!(
            client,
            user_id.clone(),
            ClientRoomEventType::MessagesAppended {
                message_ids: vec!["id-2".into()]
            }
        )
    }
    client.receive_next().await;

    // That should bump our unread count to 2
    let sidebar_items = client.sidebar.sidebar_items().await;
    assert_eq!(1, sidebar_items.len());
    assert_eq!(2, sidebar_items[0].unread_count);

    // We mark the room as read
    {
        client.expect_save_synced_room_settings(SyncedRoomSettings {
            room_id: user_id.clone().into(),
            encryption_enabled: false,
            last_read_message: Some(ArchivedMessageRef {
                stanza_id: "stanza-id-2".into(),
                // Timestamp should be rounded up…
                timestamp: Utc.with_ymd_and_hms(2024, 04, 05, 10, 00, 01).unwrap(),
            }),
        });

        event!(client, ClientEvent::SidebarChanged);
    }
    let room = client.get_room(user_id.clone()).await.to_generic_room();
    room.mark_as_read().await?;

    // This of course brings our unread count down to 0
    let sidebar_items = client.sidebar.sidebar_items().await;
    assert_eq!(1, sidebar_items.len());
    assert_eq!(0, sidebar_items[0].unread_count);

    // And we receive one final message at 2024-04-05 10:01:00.00…
    {
        time_provider.set_ymd_hms_millis(2024, 04, 05, 10, 01, 00, 00);

        recv!(
            client,
            r#"
            <message xmlns="jabber:client" from="{{OTHER_USER_ID}}/res" id="id-3" type="chat" xml:lang="en">
              <body>Hello</body>
              <stanza-id xmlns="urn:xmpp:sid:0" by="{{USER_ID}}" id="stanza-id-3" />
            </message>
            "#
        );

        event!(client, ClientEvent::SidebarChanged);
        room_event!(
            client,
            user_id.clone(),
            ClientRoomEventType::MessagesAppended {
                message_ids: vec!["id-3".into()]
            }
        )
    }
    client.receive_next().await;

    // This should bring our unread count back up to 1
    let sidebar_items = client.sidebar.sidebar_items().await;
    assert_eq!(1, sidebar_items.len());
    assert_eq!(1, sidebar_items[0].unread_count);

    // We disconnect…
    client.disconnect().await;

    // …and reconnect
    client
        .expect_login_with_strategy(
            user_id!("user@prose.org"),
            "secret",
            LoginStrategy::default().with_bookmarks_handler(|client| {
                client
                    .expect_load_bookmarks([Bookmark::direct_message(user_id!("other@prose.org"))
                        .set_sidebar_state(RoomSidebarState::InSidebar)]);

                event!(client, ClientEvent::SidebarChanged);

                // We're receiving the same messages as before from MAM but this time with
                // second-precision, instead of millisecond. Note that this leads to the second
                // being rounded up and will update our saved messages with the rounded-up
                // timestamp which effectively moves our last read message into the future.
                // When loading messages after the "last read timestamp" it brought up our already
                // read message again, but it should not count against our unread count.
                let mam_messages = [
                    MessageBuilder::new_with_id(
                        "id-1",
                        Utc.with_ymd_and_hms(2024, 04, 05, 10, 00, 01).unwrap(),
                        MessageLikePayload::message("Hello"),
                    )
                    .set_stanza_id(Some("stanza-id-1".into()))
                    .set_from(user_id!("other@prose.org"))
                    .build_archived_message("", None),
                    MessageBuilder::new_with_id(
                        "id-2",
                        Utc.with_ymd_and_hms(2024, 04, 05, 10, 00, 01).unwrap(),
                        MessageLikePayload::message("Hello"),
                    )
                    .set_stanza_id(Some("stanza-id-2".into()))
                    .set_from(user_id!("other@prose.org"))
                    .build_archived_message("", None),
                    MessageBuilder::new_with_id(
                        "id-3",
                        Utc.with_ymd_and_hms(2024, 04, 05, 10, 01, 00).unwrap(),
                        MessageLikePayload::message("Hello"),
                    )
                    .set_stanza_id(Some("stanza-id-3".into()))
                    .set_from(user_id!("other@prose.org"))
                    .build_archived_message("", None),
                ];

                client.expect_start_dm_with_strategy(
                    user_id!("other@prose.org"),
                    StartDMStrategy::default()
                        .with_load_settings_handler(|client, user_id| {
                            client.expect_load_synced_room_settings(
                                user_id.clone(),
                                Some(SyncedRoomSettings {
                                    room_id: user_id!("other@prose.org").into(),
                                    encryption_enabled: false,
                                    last_read_message: Some(ArchivedMessageRef {
                                        stanza_id: "stanza-id-2".into(),
                                        timestamp: Utc
                                            .with_ymd_and_hms(2024, 04, 05, 10, 00, 01)
                                            .unwrap(),
                                    }),
                                }),
                            )
                        })
                        .with_catch_up_handler(|client, user_id| {
                            client.expect_catchup_with_config(
                                user_id,
                                // Timestamp should be rounded up here as well…
                                Utc.with_ymd_and_hms(2024, 04, 05, 10, 00, 01).unwrap(),
                                mam_messages,
                            );
                        }),
                );

                room_event!(
                    client,
                    user_id!("other@prose.org"),
                    ClientRoomEventType::MessagesNeedReload
                );
                event!(client, ClientEvent::SidebarChanged);
            }),
        )
        .await?;

    // The unread count should be unchanged at 1
    let sidebar_items = client.sidebar.sidebar_items().await;
    assert_eq!(1, sidebar_items.len());
    assert_eq!(1, sidebar_items[0].unread_count);

    let unread_messages = room.load_unread_messages().await?.messages;
    assert_eq!(1, unread_messages.len());
    assert_eq!(Some("id-3".into()), unread_messages[0].id);

    Ok(())
}

#[mt_test]
async fn test_does_not_count_sent_messages_in_anon_muc_as_unread() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room_id = muc_id!("room@conf.prose.org");
    let our_occupant_id = client.build_occupant_id(&room_id);

    client
        .join_room_with_strategy(
            room_id,
            "anon-id",
            JoinRoomStrategy::default().with_catch_up_handler(move |client, room_id| {
                client.expect_muc_catchup_with_config(
                    room_id,
                    client.time_provider.now()
                        - Duration::seconds(client.app_config.max_catchup_duration_secs),
                    vec![
                        MessageBuilder::new_with_index(1)
                            .set_from(occupant_id!("room@conf.prose.org/friend"))
                            .build_archived_message("", None),
                        MessageBuilder::new_with_index(2)
                            .set_from(our_occupant_id.clone())
                            .build_archived_message("", None),
                        MessageBuilder::new_with_index(3)
                            .set_from(occupant_id!("room@conf.prose.org/friend"))
                            .build_archived_message("", None),
                    ],
                );
            }),
        )
        .await?;

    let sidebar_items = client.sidebar.sidebar_items().await;
    assert_eq!(1, sidebar_items.len());
    assert_eq!(2, *&sidebar_items[0].unread_count);

    Ok(())
}

#[mt_test]
async fn test_does_not_count_sent_messages_in_muc_as_unread() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room_id = muc_id!("room@conf.prose.org");
    let our_occupant_id = client.build_occupant_id(&room_id);

    client
        .join_room_with_strategy(
            room_id,
            "anon-id",
            JoinRoomStrategy::default().with_catch_up_handler(move |client, room_id| {
                client.expect_muc_catchup_with_config(
                    room_id,
                    client.time_provider.now()
                        - Duration::seconds(client.app_config.max_catchup_duration_secs),
                    vec![
                        MessageBuilder::new_with_index(1)
                            .set_from(occupant_id!("room@conf.prose.org/friend"))
                            .build_archived_message("", None),
                        MessageBuilder::new_with_index(2)
                            .set_from(our_occupant_id.clone())
                            // Specifying the anon occupant id here will help the MessageParser
                            // lookup our real JID.
                            .set_from_anon("anon-id")
                            .build_archived_message("", None),
                        MessageBuilder::new_with_index(3)
                            .set_from(occupant_id!("room@conf.prose.org/friend"))
                            .build_archived_message("", None),
                    ],
                );
            }),
        )
        .await?;

    let sidebar_items = client.sidebar.sidebar_items().await;
    assert_eq!(1, sidebar_items.len());
    assert_eq!(2, *&sidebar_items[0].unread_count);

    Ok(())
}

#[mt_test]
async fn test_does_not_count_sent_messages_in_dm_as_unread() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    client
        .start_dm_with_strategy(
            user_id!("other@prose.org"),
            StartDMStrategy::default().with_catch_up_handler(|client, user_id| {
                client.expect_catchup_with_config(
                    user_id,
                    client.time_provider.now()
                        - Duration::seconds(client.app_config.max_catchup_duration_secs),
                    vec![
                        MessageBuilder::new_with_index(1)
                            .set_from(user_id!("other@prose.org"))
                            .build_archived_message("", None),
                        MessageBuilder::new_with_index(2)
                            .set_from(user_id!("user@prose.org"))
                            .build_archived_message("", None),
                        MessageBuilder::new_with_index(3)
                            .set_from(user_id!("other@prose.org"))
                            .build_archived_message("", None),
                    ],
                )
            }),
        )
        .await?;

    let sidebar_items = client.sidebar.sidebar_items().await;
    assert_eq!(1, sidebar_items.len());
    assert_eq!(*&sidebar_items[0].unread_count, 2);

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
        last_read_message: Some(ArchivedMessageRef {
            stanza_id: MessageBuilder::stanza_id_for_index(1),
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

    client.push_ctx([
        ("OTHER_USER_ID", user_id.to_string()),
        (
            "MSG_STANZA_ID",
            MessageBuilder::stanza_id_for_index(2).to_string(),
        ),
        (
            "MSG_TIMESTAMP",
            Utc.with_ymd_and_hms(2024, 04, 26, 10, 00, 00)
                .unwrap()
                .to_rfc3339(),
        ),
    ]);
    recv!(
        client,
        r#"
      <message xmlns="jabber:client" from="{{USER_ID}}" id="X5HyLLwZYrGEJGODIb0ek4FM" to="{{USER_RESOURCE_ID}}" type="headline">
        <event xmlns="http://jabber.org/protocol/pubsub#event">
          <items node="https://prose.org/protocol/room_settings">
            <item id="{{OTHER_USER_ID}}" publisher="{{USER_ID}}">
              <room-settings xmlns="https://prose.org/protocol/room_settings" room-id="user:{{OTHER_USER_ID}}">
                <archived-message-ref xmlns="https://prose.org/protocol/archived_message_ref" stanza-id="{{MSG_STANZA_ID}}" ts="{{MSG_TIMESTAMP}}" />
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

    let strategy = StartDMStrategy::default().with_load_settings_handler(move |client, user_id| {
        client.expect_load_synced_room_settings(
            user_id.clone(),
            Some(SyncedRoomSettings {
                room_id: room_id.clone(),
                encryption_enabled: false,
                last_read_message: Some(ArchivedMessageRef {
                    stanza_id: MessageBuilder::stanza_id_for_index(2),
                    timestamp: Utc.with_ymd_and_hms(2024, 04, 26, 11, 00, 00).unwrap(),
                }),
            }),
        )
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
        last_read_message: Some(ArchivedMessageRef {
            stanza_id: MessageBuilder::stanza_id_for_index(1),
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

    client.expect_save_synced_room_settings(SyncedRoomSettings {
        room_id: room_id.clone(),
        encryption_enabled: false,
        last_read_message: Some(ArchivedMessageRef {
            stanza_id: MessageBuilder::stanza_id_for_index(3),
            timestamp: Utc.with_ymd_and_hms(2024, 04, 26, 11, 00, 00).unwrap(),
        }),
    });

    event!(client, ClientEvent::SidebarChanged);

    let room = client.get_room(room_id).await.to_generic_room();
    room.mark_as_read().await?;

    // This shouldn't do anything, since the last read message did not change.
    room.mark_as_read().await?;

    Ok(())
}

#[mt_test]
async fn test_set_unread_message_saves_settings() -> Result<()> {
    let store = store().await.expect("Failed to set up store.");

    let account = account_id!("user@prose.org");
    let muc_id = muc_id!("room@conf.prose.org");
    let room_id = RoomId::Muc(muc_id.clone());

    let mut messages = [
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
        MessageBuilder::new_with_index(4)
            .set_from(occupant_id!("room@conf.prose.org/friend"))
            .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 27, 11, 00, 00).unwrap())
            .build_message_like(),
        MessageBuilder::new_with_index(5)
            .set_from(occupant_id!("room@conf.prose.org/friend"))
            .set_timestamp(Utc.with_ymd_and_hms(2024, 04, 28, 11, 00, 00).unwrap())
            .build_message_like(),
    ];

    messages[2].stanza_id = None;
    messages[3].stanza_id = None;

    let message_repo = CachingMessageRepository::new(store.clone());
    message_repo.append(&account, &room_id, &messages).await?;

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
        last_read_message: Some(ArchivedMessageRef {
            stanza_id: MessageBuilder::stanza_id_for_index(1),
            timestamp: Utc.with_ymd_and_hms(2024, 04, 25, 10, 00, 00).unwrap(),
        }),
    });

    client
        .join_room_with_strategy(muc_id.clone(), "anon-id", join_room_strategy)
        .await?;

    client.expect_save_synced_room_settings(SyncedRoomSettings {
        room_id: room_id.clone(),
        encryption_enabled: false,
        last_read_message: Some(ArchivedMessageRef {
            stanza_id: MessageBuilder::stanza_id_for_index(2),
            timestamp: messages[1].timestamp.clone(),
        }),
    });

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::MessagesUpdated {
            message_ids: vec![
                MessageBuilder::id_for_index(1),
                MessageBuilder::id_for_index(2)
            ]
        }
    );

    event!(client, ClientEvent::SidebarChanged);

    let room = client.get_room(room_id.clone()).await.to_generic_room();
    room.set_last_read_message(&MessageBuilder::id_for_index(4))
        .await?;

    let sidebar_items = client.sidebar.sidebar_items().await;
    assert_eq!(1, sidebar_items.len());

    let sidebar_item = sidebar_items
        .get(0)
        .expect("Expected at least one SidebarItem");
    assert_eq!(3, sidebar_item.unread_count);

    client.expect_save_synced_room_settings(SyncedRoomSettings {
        room_id: room_id.clone(),
        encryption_enabled: false,
        last_read_message: Some(ArchivedMessageRef {
            stanza_id: MessageBuilder::stanza_id_for_index(5),
            timestamp: messages[4].timestamp.clone(),
        }),
    });

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::MessagesUpdated {
            message_ids: vec![
                MessageBuilder::id_for_index(2),
                MessageBuilder::id_for_index(5)
            ]
        }
    );

    event!(client, ClientEvent::SidebarChanged);

    room.set_last_read_message(&MessageBuilder::id_for_index(5))
        .await?;

    let sidebar_items = client.sidebar.sidebar_items().await;
    assert_eq!(1, sidebar_items.len());

    let sidebar_item = sidebar_items
        .get(0)
        .expect("Expected at least one SidebarItem");
    assert_eq!(0, sidebar_item.unread_count);

    Ok(())
}
