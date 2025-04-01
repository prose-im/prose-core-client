// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::tests::client::helpers::{JoinRoomStrategy, StartDMStrategy, TestClient};
use crate::{event, recv, room_event, send};
use anyhow::Result;
use chrono::Duration;
use pretty_assertions::assert_eq;
use prose_core_client::domain::shared::models::AvatarId;
use prose_core_client::dtos::*;
use prose_core_client::test::MessageBuilder;
use prose_core_client::{muc_id, occupant_id, user_id, ClientEvent, ClientRoomEventType};
use prose_proc_macros::mt_test;
use prose_xmpp::{bare, TimeProvider};

#[mt_test]
async fn test_resolves_avatars_in_dm_messages_with_avatars_received_before_start() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let msg1_id = client.get_next_message_id_with_offset(1);
    let msg2_id = client.get_next_message_id_with_offset(2);

    // Receive our avatar
    {
        recv!(
            client,
            r#"
            <message xmlns="jabber:client" from="{{USER_ID}}" id="id-1" to="{{USER_RESOURCE_ID}}" type="headline">
              <event xmlns="http://jabber.org/protocol/pubsub#event">
                <items node="urn:xmpp:avatar:metadata">
                  <item id="user-avatar-id">
                    <metadata xmlns="urn:xmpp:avatar:metadata">
                      <info bytes="20000" height="400" id="user-avatar-id" type="image/gif" width="400" />
                    </metadata>
                  </item>
                </items>
              </event>
            </message>
            "#
        );

        event!(
            client,
            ClientEvent::ContactChanged {
                ids: vec![user_id!("user@prose.org")]
            }
        );
        event!(client, ClientEvent::AccountInfoChanged);
    }
    client.receive_next().await;

    // Receive their avatar
    {
        recv!(
            client,
            r#"
            <message xmlns="jabber:client" from="them@prose.org" id="id-2" to="them@prose.org/res" type="headline">
              <event xmlns="http://jabber.org/protocol/pubsub#event">
                <items node="urn:xmpp:avatar:metadata">
                  <item id="their-avatar-id">
                    <metadata xmlns="urn:xmpp:avatar:metadata">
                      <info bytes="20000" height="400" id="their-avatar-id" type="image/gif" width="400" />
                    </metadata>
                  </item>
                </items>
              </event>
            </message>
            "#
        );

        event!(
            client,
            ClientEvent::ContactChanged {
                ids: vec![user_id!("them@prose.org")]
            }
        );
    }
    client.receive_next().await;

    // Start DM
    let room = client
        .start_dm_with_strategy(
            user_id!("them@prose.org"),
            StartDMStrategy::default().with_catch_up_handler(|client, user_id| {
                client.expect_catchup_with_config(
                    user_id,
                    client.time_provider.now()
                        - Duration::seconds(client.app_config.max_catchup_duration_secs),
                    vec![
                        MessageBuilder::new_with_index(1)
                            .set_payload("Message 1")
                            .set_from(user_id!("user@prose.org"))
                            .build_archived_message("", None),
                        MessageBuilder::new_with_index(2)
                            .set_payload("Message 2")
                            .set_from(user_id!("them@prose.org"))
                            .build_archived_message("", None),
                    ],
                );
            }),
        )
        .await?
        .to_generic_room();

    let our_avatar = Avatar {
        id: AvatarId::from_str_unchecked("user-avatar-id"),
        source: AvatarSource::Pep {
            owner: user_id!("user@prose.org").into(),
            mime_type: "image/gif".to_string(),
        },
    };
    let their_avatar = Avatar {
        id: AvatarId::from_str_unchecked("their-avatar-id"),
        source: AvatarSource::Pep {
            owner: user_id!("them@prose.org").into(),
            mime_type: "image/gif".to_string(),
        },
    };

    let messages = room.load_messages_with_ids(&[msg1_id, msg2_id]).await?;
    assert_eq!(2, messages.len());
    assert_eq!(Some(our_avatar.clone()), messages[0].from.avatar);
    assert_eq!(Some(their_avatar.clone()), messages[1].from.avatar);

    Ok(())
}

#[mt_test]
async fn test_resolves_avatars_in_dm_messages_with_avatars_received_after_start() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let msg1_id = client.get_next_message_id_with_offset(1);
    let msg2_id = client.get_next_message_id_with_offset(2);

    // Start DM
    let room = client
        .start_dm_with_strategy(
            user_id!("them@prose.org"),
            StartDMStrategy::default().with_catch_up_handler(|client, user_id| {
                client.expect_catchup_with_config(
                    user_id,
                    client.time_provider.now()
                        - Duration::seconds(client.app_config.max_catchup_duration_secs),
                    vec![
                        MessageBuilder::new_with_index(1)
                            .set_payload("Message 1")
                            .set_from(user_id!("user@prose.org"))
                            .build_archived_message("", None),
                        MessageBuilder::new_with_index(2)
                            .set_payload("Message 2")
                            .set_from(user_id!("them@prose.org"))
                            .build_archived_message("", None),
                    ],
                );
            }),
        )
        .await?
        .to_generic_room();

    // Receive our avatar
    {
        recv!(
            client,
            r#"
            <message xmlns="jabber:client" from="{{USER_ID}}" id="id-1" to="{{USER_RESOURCE_ID}}" type="headline">
              <event xmlns="http://jabber.org/protocol/pubsub#event">
                <items node="urn:xmpp:avatar:metadata">
                  <item id="user-avatar-id">
                    <metadata xmlns="urn:xmpp:avatar:metadata">
                      <info bytes="20000" height="400" id="user-avatar-id" type="image/gif" width="400" />
                    </metadata>
                  </item>
                </items>
              </event>
            </message>
            "#
        );

        event!(
            client,
            ClientEvent::ContactChanged {
                ids: vec![user_id!("user@prose.org")]
            }
        );
        event!(client, ClientEvent::AccountInfoChanged);
    }
    client.receive_next().await;

    // Receive their avatar
    {
        recv!(
            client,
            r#"
            <message xmlns="jabber:client" from="them@prose.org" id="id-2" to="them@prose.org/res" type="headline">
              <event xmlns="http://jabber.org/protocol/pubsub#event">
                <items node="urn:xmpp:avatar:metadata">
                  <item id="their-avatar-id">
                    <metadata xmlns="urn:xmpp:avatar:metadata">
                      <info bytes="20000" height="400" id="their-avatar-id" type="image/gif" width="400" />
                    </metadata>
                  </item>
                </items>
              </event>
            </message>
            "#
        );

        event!(
            client,
            ClientEvent::ContactChanged {
                ids: vec![user_id!("them@prose.org")]
            }
        );
        event!(client, ClientEvent::SidebarChanged);
    }
    client.receive_next().await;

    let our_avatar = Avatar {
        id: AvatarId::from_str_unchecked("user-avatar-id"),
        source: AvatarSource::Pep {
            owner: user_id!("user@prose.org").into(),
            mime_type: "image/gif".to_string(),
        },
    };
    let their_avatar = Avatar {
        id: AvatarId::from_str_unchecked("their-avatar-id"),
        source: AvatarSource::Pep {
            owner: user_id!("them@prose.org").into(),
            mime_type: "image/gif".to_string(),
        },
    };

    let messages = room.load_messages_with_ids(&[msg1_id, msg2_id]).await?;
    assert_eq!(2, messages.len());
    assert_eq!(Some(our_avatar.clone()), messages[0].from.avatar);
    assert_eq!(Some(their_avatar.clone()), messages[1].from.avatar);

    Ok(())
}

#[mt_test]
async fn test_resolves_avatars_in_muc() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let msg1_id = client.get_next_message_id_with_offset(1);
    let msg2_id = client.get_next_message_id_with_offset(2);

    let strategy = JoinRoomStrategy::default().with_catch_up_handler(|client, user_id| {
        client.expect_muc_catchup_with_config(
            user_id,
            client.time_provider.now()
                - Duration::seconds(client.app_config.max_catchup_duration_secs),
            vec![
                MessageBuilder::new_with_index(1)
                    .set_payload("Message 1")
                    .set_from(occupant_id!("room@conference.prose.org/user"))
                    .build_archived_message("", None),
                MessageBuilder::new_with_index(2)
                    .set_payload("Message 2")
                    .set_from(occupant_id!("room@conference.prose.org/them"))
                    .build_archived_message("", None),
            ],
        );
    });

    client
        .join_room_with_strategy(muc_id!("room@conference.prose.org"), "anon-id", strategy)
        .await?;

    {
        recv!(
            client,
            r#"
        <presence xmlns="jabber:client" from="room@conference.prose.org/user" to="{{USER_RESOURCE_ID}}" xml:lang="en">
          <x xmlns="vcard-temp:x:update">
            <photo>0000000000000000000000000000000000000001</photo>
          </x>
          <x xmlns="http://jabber.org/protocol/muc#user">
            <status code="100" />
            <item affiliation="none" jid="{{USER_RESOURCE_ID}}" role="participant" />
            <status code="110" />
          </x>
        </presence>
        "#
        );

        room_event!(
            client,
            muc_id!("room@conference.prose.org"),
            ClientRoomEventType::ParticipantsChanged
        );
        room_event!(
            client,
            muc_id!("room@conference.prose.org"),
            ClientRoomEventType::ParticipantsChanged
        );
    }
    client.receive_next().await;

    {
        recv!(
            client,
            r#"
        <presence xmlns="jabber:client" from="room@conference.prose.org/them" to="{{USER_RESOURCE_ID}}" xml:lang="en">
          <x xmlns="vcard-temp:x:update">
            <photo>0000000000000000000000000000000000000002</photo>
          </x>
          <x xmlns="http://jabber.org/protocol/muc#user">
            <item affiliation="none" jid="them@prose.org/res" role="participant" />
          </x>
        </presence>
        "#
        );

        room_event!(
            client,
            muc_id!("room@conference.prose.org"),
            ClientRoomEventType::ParticipantsChanged
        );
        room_event!(
            client,
            muc_id!("room@conference.prose.org"),
            ClientRoomEventType::ParticipantsChanged
        );
    }
    client.receive_next().await;

    let room = client
        .get_room(muc_id!("room@conference.prose.org"))
        .await
        .to_generic_room();

    let our_avatar = Avatar {
        id: AvatarId::from_str_unchecked("0000000000000000000000000000000000000001"),
        source: AvatarSource::Vcard {
            owner: occupant_id!("room@conference.prose.org/user").into(),
            real_id: Some(user_id!("user@prose.org")),
        },
    };
    let their_avatar = Avatar {
        id: AvatarId::from_str_unchecked("0000000000000000000000000000000000000002"),
        source: AvatarSource::Vcard {
            owner: occupant_id!("room@conference.prose.org/them").into(),
            real_id: Some(user_id!("them@prose.org")),
        },
    };

    let messages = room.load_messages_with_ids(&[msg1_id, msg2_id]).await?;
    assert_eq!(2, messages.len());
    assert_eq!(Some(our_avatar.clone()), messages[0].from.avatar);
    assert_eq!(Some(their_avatar.clone()), messages[1].from.avatar);

    Ok(())
}
