// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use pretty_assertions::assert_eq;

use prose_core_client::domain::rooms::models::RoomSidebarState;
use prose_core_client::domain::shared::models::{MucId, OccupantId, RoomType, UserId};
use prose_core_client::dtos::{Bookmark, RoomId};
use prose_core_client::test::MessageBuilder;
use prose_core_client::{
    muc_id, occupant_id, user_id, ClientEvent, ClientRoomEventType, ConnectionEvent,
};
use prose_proc_macros::mt_test;
use prose_xmpp::TimeProvider;

use crate::tests::client::helpers::{JoinRoomStrategy, LoginStrategy, StartDMStrategy, TestClient};
use crate::{event, recv, room_event, send};

#[mt_test]
async fn test_reconnect_catches_up_rooms() -> anyhow::Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login_with_strategy(
            user_id!("user@prose.org"),
            "secret",
            LoginStrategy::default().with_bookmarks_handler(|client| {
                client.expect_load_bookmarks([
                    Bookmark::direct_message(user_id!("other@prose.org"))
                        .set_sidebar_state(RoomSidebarState::InSidebar),
                    Bookmark::group(muc_id!("group@conf.prose.org"), "My Group")
                        .set_sidebar_state(RoomSidebarState::InSidebar),
                ]);

                event!(client, ClientEvent::SidebarChanged);

                client.expect_start_dm_with_strategy(
                    user_id!("other@prose.org"),
                    StartDMStrategy::default(),
                );

                event!(client, ClientEvent::SidebarChanged);

                client.expect_join_room_with_strategy(
                    muc_id!("group@conf.prose.org"),
                    "anon-id",
                    JoinRoomStrategy::default()
                        .with_room_name("My Group")
                        .with_room_type(RoomType::Group),
                );

                event!(client, ClientEvent::SidebarChanged);
            }),
        )
        .await?;

    assert_eq!(2, client.sidebar.sidebar_items().await.len());

    event!(
        client,
        ClientEvent::ConnectionStatusChanged {
            event: ConnectionEvent::Disconnect { error: None }
        }
    );

    client.simulate_disconnect().await;

    client
        .expect_login_with_strategy(
            user_id!("user@prose.org"),
            "secret",
            LoginStrategy::default().with_bookmarks_handler(|client| {
                client.expect_load_bookmarks([
                    Bookmark::direct_message(user_id!("other@prose.org"))
                        .set_sidebar_state(RoomSidebarState::InSidebar),
                    Bookmark::group(muc_id!("group@conf.prose.org"), "My Group")
                        .set_sidebar_state(RoomSidebarState::InSidebar),
                ]);

                event!(client, ClientEvent::SidebarChanged);

                client.expect_start_dm_with_strategy(
                    user_id!("other@prose.org"),
                    StartDMStrategy::default().with_catch_up_handler(|client, user_id| {
                        client.expect_catchup_with_config(
                            user_id,
                            client.time_provider.now(),
                            vec![
                                MessageBuilder::new_with_index(1)
                                    .set_from(user_id!("other@prose.org"))
                                    .build_archived_message("", None),
                                MessageBuilder::new_with_index(2)
                                    .set_from(user_id!("other@prose.org"))
                                    .build_archived_message("", None),
                            ],
                        );
                    }),
                );

                room_event!(
                    client,
                    user_id!("other@prose.org"),
                    ClientRoomEventType::MessagesNeedReload
                );
                event!(client, ClientEvent::SidebarChanged);

                client.expect_join_room_with_strategy(
                    muc_id!("group@conf.prose.org"),
                    "anon-id",
                    JoinRoomStrategy::default()
                        .with_room_name("My Group")
                        .with_room_type(RoomType::Group)
                        .with_catch_up_handler(|client, room_id| {
                            client.expect_muc_catchup_with_config(
                                room_id,
                                client.time_provider.now(),
                                vec![MessageBuilder::new_with_index(1)
                                    .set_from(occupant_id!("group@conf.prose.org/other"))
                                    .build_archived_message("", None)],
                            );
                        }),
                );

                room_event!(
                    client,
                    muc_id!("group@conf.prose.org"),
                    ClientRoomEventType::MessagesNeedReload
                );
                event!(client, ClientEvent::SidebarChanged);
            }),
        )
        .await?;

    let sidebar_items = client.sidebar.sidebar_items().await;

    assert_eq!(2, sidebar_items.len());

    let dm_item = sidebar_items
        .iter()
        .find(|item| {
            item.room.to_generic_room().jid() == &RoomId::User(user_id!("other@prose.org"))
        })
        .unwrap();

    let group_item = sidebar_items
        .iter()
        .find(|item| {
            item.room.to_generic_room().jid() == &RoomId::Muc(muc_id!("group@conf.prose.org"))
        })
        .unwrap();

    assert_eq!(2, dm_item.unread_count);
    assert_eq!(1, group_item.unread_count);

    Ok(())
}

#[mt_test]
async fn test_reconnects_muc_room_after_failed_self_ping() -> anyhow::Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login_with_strategy(
            user_id!("user@prose.org"),
            "secret",
            LoginStrategy::default().with_bookmarks_handler(|client| {
                client.expect_load_bookmarks([Bookmark::group(
                    muc_id!("group@conf.prose.org"),
                    "My Group",
                )
                .set_sidebar_state(RoomSidebarState::InSidebar)]);

                event!(client, ClientEvent::SidebarChanged);

                client.expect_join_room_with_strategy(
                    muc_id!("group@conf.prose.org"),
                    "anon-id",
                    JoinRoomStrategy::default()
                        .with_room_name("My Group")
                        .with_room_type(RoomType::Group),
                );

                event!(client, ClientEvent::SidebarChanged);
            }),
        )
        .await?;

    send!(
        client,
        r#"
        <iq xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" type="get">
          <ping xmlns="urn:xmpp:ping" />
        </iq>
        "#
    );

    recv!(
        client,
        r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="result" />
        "#
    );

    let occupant_id = client.build_occupant_id(&muc_id!("group@conf.prose.org"));
    client.push_ctx([("OCCUPANT_ID", occupant_id.to_string())]);

    send!(
        client,
        r#"
        <iq xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="{{OCCUPANT_ID}}" type="get">
          <ping xmlns="urn:xmpp:ping" />
        </iq>
        "#
    );

    recv!(
        client,
        r#"
        <iq xmlns="jabber:client" id="{{ID}}" type="error">
          <error type="cancel">
            <not-acceptable xmlns="urn:ietf:params:xml:ns:xmpp-stanzas" />
          </error>
        </iq>
        "#
    );

    client.pop_ctx();

    event!(client, ClientEvent::SidebarChanged);

    client.expect_join_room_with_strategy(
        muc_id!("group@conf.prose.org"),
        "anon-id",
        JoinRoomStrategy::default()
            .with_room_name("My Group")
            .with_room_type(RoomType::Group)
            .with_catch_up_handler(|client, room_id| {
                client.expect_muc_catchup_with_config(
                    room_id,
                    client.time_provider.now(),
                    vec![MessageBuilder::new_with_index(1)
                        .set_from(occupant_id!("group@conf.prose.org/other"))
                        .build_archived_message("", None)],
                );
            })
            .with_vcard_handler(|_, _, _| {
                // User Infos remain cached
            }),
    );

    room_event!(
        client,
        muc_id!("group@conf.prose.org"),
        ClientRoomEventType::MessagesNeedReload
    );
    event!(client, ClientEvent::SidebarChanged);

    client.simulate_ping_timer().await;

    Ok(())
}
