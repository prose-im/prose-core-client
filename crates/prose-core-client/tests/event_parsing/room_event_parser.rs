// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::app::event_handlers::{
    OccupantEvent, OccupantEventType, RoomEvent, RoomEventType, ServerEvent, UserStatusEvent,
    UserStatusEventType,
};
use prose_core_client::domain::rooms::models::RoomAffiliation;
use prose_core_client::domain::shared::models::AnonOccupantId;
use prose_core_client::dtos::*;
use prose_core_client::test::parse_xml;
use prose_core_client::{muc_id, occupant_id, user_id, user_resource_id};
use prose_proc_macros::mt_test;

#[mt_test]
async fn test_room_topic_changed() -> Result<()> {
    // Room Subject (https://xmpp.org/extensions/xep-0045.html#enter-subject)
    let events = parse_xml(
        r#"
        <message xmlns='jabber:client' from='room@prose.org' type='groupchat'>
            <subject>Fire Burn and Cauldron Bubble!</subject>
        </message>"#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::Room(RoomEvent {
            room_id: muc_id!("room@prose.org"),
            r#type: RoomEventType::RoomTopicChanged {
                new_topic: Some("Fire Burn and Cauldron Bubble!".to_string())
            },
        })]
    );

    let events = parse_xml(
        r#"
        <message xmlns='jabber:client' from='room@prose.org' type='groupchat'>
            <subject />
        </message>"#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::Room(RoomEvent {
            room_id: muc_id!("room@prose.org").into(),
            r#type: RoomEventType::RoomTopicChanged { new_topic: None },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_room_destroyed() -> Result<()> {
    // Destroying a Room (https://xmpp.org/extensions/xep-0045.html#destroyroom)
    let events =
        parse_xml(
            r#"
        <presence xmlns="jabber:client" from="room@prose.org/nick" type="unavailable">
          <occupant-id xmlns="urn:xmpp:occupant-id:0" id="P394Wptcfdr7IJxWI28YYCBdAAvQkax+0dEkWR/r5CY=" />
          <x xmlns="http://jabber.org/protocol/muc#user">
            <destroy jid="new-room@prose.org" />
            <item affiliation="owner" jid="nick@prose.org/res" role="none" />
            <status code="110" />
          </x>
        </presence>
      "#,
        )
        .await?;

    assert_eq!(
        events,
        vec![ServerEvent::Room(RoomEvent {
            room_id: muc_id!("room@prose.org").into(),
            r#type: RoomEventType::Destroyed {
                replacement: Some(muc_id!("new-room@prose.org"))
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_room_config_changed() -> Result<()> {
    // Notification of Configuration Changes (https://xmpp.org/extensions/xep-0045.html#roomconfig-notify)
    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="room@prose.org" type="groupchat">
            <x xmlns="http://jabber.org/protocol/muc#user">
                <status code="104" />
            </x>
        </message>
      "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::Room(RoomEvent {
            room_id: muc_id!("room@prose.org").into(),
            r#type: RoomEventType::RoomConfigChanged,
        })]
    );

    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="room@prose.org" type="groupchat">
            <x xmlns="http://jabber.org/protocol/muc#user">
                <status code="104" />
                <status code="170" />
            </x>
        </message>
      "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::Room(RoomEvent {
            room_id: muc_id!("room@prose.org").into(),
            r#type: RoomEventType::RoomConfigChanged,
        })]
    );

    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="room@prose.org" type="groupchat">
            <x xmlns="http://jabber.org/protocol/muc#user">
                <status code="172" />
            </x>
        </message>
      "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::Room(RoomEvent {
            room_id: muc_id!("room@prose.org").into(),
            r#type: RoomEventType::RoomConfigChanged,
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_user_was_permanently_removed() -> Result<()> {
    // Kicking an Occupant (https://xmpp.org/extensions/xep-0045.html#kick)
    let events = parse_xml(
        r#"
        <presence xmlns='jabber:client' from='room@prose.org/nick' type='unavailable'>
            <x xmlns='http://jabber.org/protocol/muc#user'>
                <item affiliation='none' role='none'/>
                <status code='307'/>
            </x>
        </presence>
      "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![
            ServerEvent::UserStatus(UserStatusEvent {
                user_id: occupant_id!("room@prose.org/nick").into(),
                r#type: UserStatusEventType::AvailabilityChanged {
                    availability: Availability::Unavailable,
                    priority: 0
                },
            }),
            ServerEvent::Occupant(OccupantEvent {
                occupant_id: occupant_id!("room@prose.org/nick"),
                anon_occupant_id: None,
                real_id: None,
                is_self: false,
                r#type: OccupantEventType::PermanentlyRemoved
            })
        ]
    );

    Ok(())
}

#[mt_test]
async fn test_user_was_disconnected_by_server() -> Result<()> {
    // Service removes user because of service shut down (https://xmpp.org/extensions/xep-0045.html#service-shutdown-kick)
    let events = parse_xml(
        r#"
        <presence xmlns="jabber:client" from="room@prose.org/nick" type="unavailable">
            <x xmlns="http://jabber.org/protocol/muc#user">
                <item affiliation="member" jid="user@prose.org/res" role="none" />
                <status code='110'/>
                <status code='332'/>
            </x>
        </presence>
      "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![
            ServerEvent::UserStatus(UserStatusEvent {
                user_id: occupant_id!("room@prose.org/nick").into(),
                r#type: UserStatusEventType::AvailabilityChanged {
                    availability: Availability::Unavailable,
                    priority: 0
                },
            }),
            ServerEvent::Occupant(OccupantEvent {
                occupant_id: occupant_id!("room@prose.org/nick"),
                anon_occupant_id: None,
                real_id: Some(user_id!("user@prose.org")),
                is_self: true,
                r#type: OccupantEventType::DisconnectedByServer,
            })
        ]
    );

    Ok(())
}

#[mt_test]
async fn test_user_entered_room() -> Result<()> {
    // Entering a room (https://xmpp.org/extensions/xep-0045.html#example-21)
    let events = parse_xml(
        r#"
        <presence xmlns="jabber:client" from="room@prose.org/nick">
            <x xmlns="vcard-temp:x:update">
                <photo>cdc05cb9c48d5e817a36d462fe0470a0579e570a</photo>
            </x>
            <occupant-id xmlns="urn:xmpp:occupant-id:0" id="gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw=" />
            <x xmlns="http://jabber.org/protocol/muc#user">
                <item affiliation="none" jid="user@prose.org/res" role="participant" />
            </x>
        </presence>
      "#,
    )
        .await?;

    assert_eq!(
        events,
        vec![
            ServerEvent::UserStatus(UserStatusEvent {
                user_id: occupant_id!("room@prose.org/nick").into(),
                r#type: UserStatusEventType::AvailabilityChanged {
                    availability: Availability::Available,
                    priority: 0
                },
            }),
            ServerEvent::Occupant(OccupantEvent {
                occupant_id: occupant_id!("room@prose.org/nick"),
                anon_occupant_id: Some(AnonOccupantId::from(
                    "gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw="
                )),
                real_id: Some(user_id!("user@prose.org")),
                is_self: false,
                r#type: OccupantEventType::AffiliationChanged {
                    affiliation: RoomAffiliation::None
                },
            }),
        ]
    );

    Ok(())
}

#[mt_test]
async fn test_affiliation_change_with_multiple_resources() -> Result<()> {
    // https://xmpp.org/extensions/xep-0045.html#enter-conflict

    // If a user joins a room with the same nickname from multiple resources, the resources are
    // merged into a single presence. If one resource goes offline again, we won't receive a
    // "unavailable" presence but another affiliation change with the affected item removed.
    // For our intents and purposes we'll assume that the affiliation, role and bare jid of all
    // resources are identical and ignore all but the first one in the list.

    let events = parse_xml(
        r#"
        <presence xmlns='jabber:client' from="room@prose.org/nick">
            <x xmlns='vcard-temp:x:update'>
                <photo>cdc05cb9c48d5e817a36d462fe0470a0579e570a</photo>
            </x>
            <occupant-id xmlns='urn:xmpp:occupant-id:0' id="gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw=" />
            <x xmlns='http://jabber.org/protocol/muc#user'>
                <item affiliation="none" jid="user@prose.org/res1" role="participant" />
                <item affiliation="none" jid="user@prose.org/res2" role="participant" />
            </x>
        </presence>
      "#,
    )
        .await?;

    assert_eq!(
        events,
        vec![
            ServerEvent::UserStatus(UserStatusEvent {
                user_id: occupant_id!("room@prose.org/nick").into(),
                r#type: UserStatusEventType::AvailabilityChanged {
                    availability: Availability::Available,
                    priority: 0
                },
            }),
            ServerEvent::Occupant(OccupantEvent {
                occupant_id: occupant_id!("room@prose.org/nick"),
                anon_occupant_id: Some(AnonOccupantId::from(
                    "gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw="
                )),
                real_id: Some(user_id!("user@prose.org")),
                is_self: false,
                r#type: OccupantEventType::AffiliationChanged {
                    affiliation: RoomAffiliation::None
                },
            }),
        ]
    );

    Ok(())
}

#[mt_test]
async fn test_user_exited_room() -> Result<()> {
    // Exiting a Room (https://xmpp.org/extensions/xep-0045.html#exit)
    let events = parse_xml(
        r#"
        <presence xmlns="jabber:client" from="room@prose.org/nick" type="unavailable">
            <status>Disconnected: closed</status>
            <occupant-id xmlns="urn:xmpp:occupant-id:0" id="FvcD+GDkmT8LQAb55uozvL7cZCTBjz3VgQfAcSLtrkM=" />
            <x xmlns="http://jabber.org/protocol/muc#user">
                <item affiliation="member" jid="user@prose.org/res" role="none" />
            </x>
        </presence>
      "#,
    )
        .await?;

    assert_eq!(
        events,
        vec![ServerEvent::UserStatus(UserStatusEvent {
            user_id: occupant_id!("room@prose.org/nick").into(),
            r#type: UserStatusEventType::AvailabilityChanged {
                availability: Availability::Unavailable,
                priority: 0
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_received_invite() -> Result<()> {
    // Mediated invitation (https://xmpp.org/extensions/xep-0045.html#invite-mediated)
    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="room@prose.org">
            <x xmlns='http://jabber.org/protocol/muc#user'>
                <invite from='user@prose.org/res'>
                    <reason>Hey Hecate, this is the place for all good witches!</reason>
                </invite>
                <password>cauldronburn</password>
            </x>
        </message>
      "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::Room(RoomEvent {
            room_id: muc_id!("room@prose.org").into(),
            r#type: RoomEventType::ReceivedInvitation {
                sender: user_resource_id!("user@prose.org/res"),
                password: Some("cauldronburn".to_string())
            },
        })]
    );

    // Direct invitation (https://xmpp.org/extensions/xep-0249.html)
    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="user@prose.org/res">
            <x xmlns="jabber:x:conference" 
                jid="room@prose.org" 
                password="cauldronburn" 
                reason="Hey Hecate, this is the place for all good witches!" 
            />
        </message>
      "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::Room(RoomEvent {
            room_id: muc_id!("room@prose.org").into(),
            r#type: RoomEventType::ReceivedInvitation {
                sender: user_resource_id!("user@prose.org/res"),
                password: Some("cauldronburn".to_string())
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_user_was_invited() -> Result<()> {
    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="room@groups.prose.org">
            <x xmlns="http://jabber.org/protocol/muc#user">
                <item affiliation="member" jid="user2@prose.org">
                    <reason>Invited by user1@prose.org/res</reason>
                </item>
            </x>
        </message>
      "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::Room(RoomEvent {
            room_id: muc_id!("room@groups.prose.org").into(),
            r#type: RoomEventType::UserAdded {
                user_id: user_id!("user2@prose.org"),
                affiliation: RoomAffiliation::Member,
                reason: Some("Invited by user1@prose.org/res".to_string()),
            },
        })]
    );

    Ok(())
}
