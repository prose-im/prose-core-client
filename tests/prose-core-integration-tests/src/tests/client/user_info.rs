// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use pretty_assertions::assert_eq;
use xmpp_parsers::roster;
use xmpp_parsers::roster::Item as RosterItem;

use prose_core_client::dtos::{
    Availability, Avatar, AvatarSource, Contact, Group, MucId, ParticipantInfo,
    PresenceSubscription, RoomAffiliation, UserId,
};
use prose_core_client::{muc_id, user_id, ClientEvent, ClientRoomEventType};
use prose_proc_macros::mt_test;
use prose_xmpp::bare;
use prose_xmpp::stanza::{vcard4, VCard4};

use crate::{event, recv, room_event};

use super::helpers::{JoinRoomStrategy, LoginStrategy, TestClient};

// 1:1 chat
// MUC
// Events

#[mt_test]
async fn test_aggregates_user_data() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login_with_strategy(
            user_id!("user@prose.org"),
            "secret",
            LoginStrategy::default().with_roster_items([
                RosterItem {
                    jid: bare!("friend@prose.org"),
                    name: Some("Jimmy".to_string()),
                    subscription: roster::Subscription::Both,
                    ask: Default::default(),
                    groups: vec![roster::Group("Team".to_string())],
                },
                RosterItem {
                    jid: bare!("unknown@prose.org"),
                    name: None,
                    subscription: roster::Subscription::Both,
                    ask: Default::default(),
                    groups: vec![roster::Group("Team".to_string())],
                },
            ]),
        )
        .await?;

    let contacts = client.contact_list.load_contacts().await?;

    assert_eq!(
        vec![
            Contact {
                id: user_id!("friend@prose.org"),
                name: "Jimmy".to_string(),
                full_name: None,
                avatar: None,
                availability: Default::default(),
                status: None,
                group: Group::Team,
                presence_subscription: PresenceSubscription::Mutual,
            },
            Contact {
                id: user_id!("unknown@prose.org"),
                name: "Unknown".to_string(),
                full_name: None,
                avatar: None,
                availability: Default::default(),
                status: None,
                group: Group::Team,
                presence_subscription: PresenceSubscription::Mutual,
            }
        ],
        contacts
    );

    {
        client.receive_vcard(
            &user_id!("friend@prose.org"),
            VCard4 {
                n: vec![vcard4::Name {
                    surname: Some("Shmoe".to_string()),
                    given: Some("Jim".to_string()),
                    additional: None,
                }],
                ..Default::default()
            },
        );

        event!(
            client,
            ClientEvent::ContactChanged {
                ids: vec![user_id!("friend@prose.org")]
            }
        );
    }
    client.receive_next().await;

    let contacts = client.contact_list.load_contacts().await?;

    assert_eq!(
        vec![
            Contact {
                id: user_id!("friend@prose.org"),
                name: "Jim Shmoe".to_string(),
                full_name: Some("Jim Shmoe".to_string()),
                avatar: None,
                availability: Default::default(),
                status: None,
                group: Group::Team,
                presence_subscription: PresenceSubscription::Mutual,
            },
            Contact {
                id: user_id!("unknown@prose.org"),
                name: "Unknown".to_string(),
                full_name: None,
                avatar: None,
                availability: Default::default(),
                status: None,
                group: Group::Team,
                presence_subscription: PresenceSubscription::Mutual,
            }
        ],
        contacts
    );

    {
        client.receive_nickname(&user_id!("friend@prose.org"), "Jimmy S.");

        event!(
            client,
            ClientEvent::ContactChanged {
                ids: vec![user_id!("friend@prose.org")]
            }
        );
    }
    client.receive_next().await;

    let contacts = client.contact_list.load_contacts().await?;

    assert_eq!(
        vec![
            Contact {
                id: user_id!("friend@prose.org"),
                name: "Jimmy S.".to_string(),
                full_name: Some("Jim Shmoe".to_string()),
                avatar: None,
                availability: Default::default(),
                status: None,
                group: Group::Team,
                presence_subscription: PresenceSubscription::Mutual,
            },
            Contact {
                id: user_id!("unknown@prose.org"),
                name: "Unknown".to_string(),
                full_name: None,
                avatar: None,
                availability: Default::default(),
                status: None,
                group: Group::Team,
                presence_subscription: PresenceSubscription::Mutual,
            }
        ],
        contacts
    );

    Ok(())
}

#[mt_test]
async fn test_joins_room() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room_id = muc_id!("room@conference.prose.org");

    let strategy = JoinRoomStrategy::default().with_occupant_presences_handler(|client, _room| {
        recv!(
            client,
            r#"
            <presence xmlns="jabber:client" from="{{ROOM_ID}}/user1" to="{{USER_RESOURCE_ID}}" xml:lang="en">
              <nick xmlns="http://jabber.org/protocol/nick">John Shmoe</nick>
              <c xmlns="http://jabber.org/protocol/caps" hash="sha-1" node="https://cheogram.com" ver="hAx0qhppW5/ZjrpXmbXW0F2SJVM=" />
              <x xmlns="vcard-temp:x:update">
                <photo>ff04ffad2762376b8de08504ced6260553b15eed</photo>
              </x>
              <x xmlns="http://jabber.org/protocol/muc#user">
                <item affiliation="member" jid="john@prose.org/res" role="participant" />
              </x>
              <show>away</show>
            </presence>
            "#
        )
    }).with_members(
        [user_id!("john@prose.org")]
    );

    client
        .join_room_with_strategy(room_id.clone(), "anon-id", strategy)
        .await?;

    let room = client.get_room(room_id.clone()).await.to_generic_room();

    let mut participants = room.participants();
    participants.sort_by(|p1, p2| p1.name.cmp(&p2.name));

    let john = ParticipantInfo {
        id: room_id.occupant_id_with_nickname("user1")?.into(),
        user_id: Some(user_id!("john@prose.org")),
        name: "John Shmoe".to_string(),
        is_self: false,
        availability: Availability::Away,
        affiliation: RoomAffiliation::Member,
        avatar: Some(Avatar {
            id: "ff04ffad2762376b8de08504ced6260553b15eed".parse()?,
            source: AvatarSource::Vcard {
                owner: room_id.occupant_id_with_nickname("user1")?.into(),
                real_id: Some(user_id!("john@prose.org")),
            },
        }),
        client: Some("https://cheogram.com".parse()?),
        status: None,
    };
    let user = ParticipantInfo {
        id: client.build_occupant_id(&room_id).into(),
        user_id: Some(user_id!("user@prose.org")),
        name: "Jane Doe".to_string(),
        is_self: true,
        availability: Availability::Available,
        affiliation: RoomAffiliation::Owner,
        avatar: None,
        client: Some("https://prose.org".parse()?),
        status: None,
    };

    assert_eq!(vec![user.clone(), john.clone()], participants);

    client.push_ctx([("ROOM_ID", room_id.to_string())]);

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
            user,
            ParticipantInfo {
                id: room_id.occupant_id_with_nickname("user2")?.into(),
                user_id: Some(user_id!("jim@prose.org")),
                name: "Jim Doe".to_string(),
                is_self: false,
                availability: Availability::Available,
                affiliation: RoomAffiliation::None,
                avatar: Some(Avatar {
                    id: "ff04ffad2762376b8de08504ced6260553b15eed".parse()?,
                    source: AvatarSource::Vcard {
                        owner: room_id.occupant_id_with_nickname("user2")?.into(),
                        real_id: user_id!("jim@prose.org").into(),
                    },
                }),
                client: Some("http://conversations.im".parse()?),
                status: None,
            },
            john,
        ],
        participants
    );

    Ok(())
}
