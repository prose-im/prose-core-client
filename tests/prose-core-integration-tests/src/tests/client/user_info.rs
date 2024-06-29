// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use pretty_assertions::assert_eq;

use prose_core_client::dtos::{
    Availability, Avatar, AvatarSource, MucId, ParticipantInfo, RoomAffiliation, UserId,
};
use prose_core_client::{muc_id, user_id, ClientRoomEventType};
use prose_proc_macros::mt_test;

use crate::{recv, room_event};

use super::helpers::{JoinRoomStrategy, TestClient};

// 1:1 chat
// MUC
// Events

#[mt_test]
async fn test_joins_room() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room_id = muc_id!("room@conference.prose.org");

    let strategy = JoinRoomStrategy::default().with_occupant_presences_handler(|client, room| {
        recv!(
            client,
            r#"
            <presence xmlns="jabber:client" from="{{ROOM_ID}}/user1" to="{{USER_RESOURCE_ID}}" xml:lang="en">
              <nick xmlns="http://jabber.org/protocol/nick">Jane Doe</nick>
              <c xmlns="http://jabber.org/protocol/caps" hash="sha-1" node="https://cheogram.com" ver="hAx0qhppW5/ZjrpXmbXW0F2SJVM=" />
              <x xmlns="vcard-temp:x:update">
                <photo>ff04ffad2762376b8de08504ced6260553b15eed</photo>
              </x>
              <x xmlns="http://jabber.org/protocol/muc#user">
                <item affiliation="member" jid="jane@prose.org/res" role="participant" />
              </x>
              <show>away</show>
            </presence>
            "#
        )
    }).with_members(
        [user_id!("jane@prose.org")]
    ).with_vcard_handler(|client, room_id, user_id| {
        (JoinRoomStrategy::default().expect_load_vcard)(client, room_id, user_id);

        client.expect_load_vcard(&user_id!("jane@prose.org"));
        client.receive_not_found_iq_response();
    });

    client
        .join_room_with_strategy(room_id.clone(), "anon-id", strategy)
        .await?;

    let room = client.get_room(room_id.clone()).await.to_generic_room();

    let mut participants = room.participants();
    participants.sort_by(|p1, p2| p1.name.cmp(&p2.name));

    let jane = ParticipantInfo {
        id: Some(user_id!("jane@prose.org")),
        name: "Jane Doe".to_string(),
        is_self: false,
        availability: Availability::Away,
        affiliation: RoomAffiliation::Member,
        avatar: Some(Avatar {
            id: "ff04ffad2762376b8de08504ced6260553b15eed".parse()?,
            source: AvatarSource::Vcard,
            owner: user_id!("jane@prose.org").into(),
        }),
        client: Some("https://cheogram.com".parse()?),
    };
    let user = ParticipantInfo {
        id: Some(user_id!("user@prose.org")),
        name: "Joe".to_string(),
        is_self: true,
        availability: Availability::Available,
        affiliation: RoomAffiliation::Owner,
        avatar: None,
        client: Some("https://prose.org".parse()?),
    };

    assert_eq!(vec![jane.clone(), user.clone()], participants);

    client.push_ctx([("ROOM_ID".into(), room_id.to_string())].into());

    recv!(
        client,
        r#"
        <presence xmlns="jabber:client" from="{{ROOM_ID}}/user2" to="{{USER_RESOURCE_ID}}" xml:lang="en">
          <nick xmlns="http://jabber.org/protocol/nick">Jim Doe</nick>
          <c xmlns="http://jabber.org/protocol/caps" hash="sha-1" node="http://conversations.im" ver="hAx0qhppW5/ZjrpXmbXW0F2SJVM=" />
          <x xmlns="vcard-temp:x:update">
            <photo>ff04ffad2762376b8de08504ced6260553b15eed</photo>
          </x>
          <x xmlns="http://jabber.org/protocol/muc#user">
            <item affiliation="none" jid="jim@prose.org/res" role="participant" />
          </x>
        </presence>
        "#
    );

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::ParticipantsChanged
    );

    client.expect_load_vcard(&user_id!("jim@prose.org"));
    client.receive_not_found_iq_response();

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::ParticipantsChanged
    );

    client.receive_next().await;

    let mut participants = room.participants();
    participants.sort_by(|p1, p2| p1.name.cmp(&p2.name));

    assert_eq!(
        vec![
            jane,
            ParticipantInfo {
                id: Some(user_id!("jim@prose.org")),
                name: "Jim Doe".to_string(),
                is_self: false,
                availability: Availability::Available,
                affiliation: RoomAffiliation::None,
                avatar: Some(Avatar {
                    id: "ff04ffad2762376b8de08504ced6260553b15eed".parse()?,
                    source: AvatarSource::Vcard,
                    owner: user_id!("jim@prose.org").into(),
                }),
                client: Some("http://conversations.im".parse()?),
            },
            user
        ],
        participants
    );

    Ok(())
}
