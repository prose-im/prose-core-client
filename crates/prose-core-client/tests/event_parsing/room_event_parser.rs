// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::domain::rooms::models::{ComposeState, RoomAffiliation};
use prose_core_client::domain::shared::models::{
    RoomEvent, RoomEventType, RoomUserInfo, ServerEvent,
};
use prose_core_client::dtos::{Availability, RoomId};
use prose_core_client::room_id;
use prose_core_client::test::parse_xml;
use prose_proc_macros::mt_test;
use prose_xmpp::full;

#[mt_test]
async fn test_room_topic_changed() -> Result<()> {
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
            room_id: room_id!("room@prose.org"),
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
            room_id: room_id!("room@prose.org"),
            r#type: RoomEventType::RoomTopicChanged { new_topic: None },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_room_destroyed() -> Result<()> {
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
            room_id: room_id!("room@prose.org"),
            r#type: RoomEventType::RoomWasDestroyed {
                alternate_room: Some(room_id!("new-room@prose.org"))
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_room_config_changed() -> Result<()> {
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
            room_id: room_id!("room@prose.org"),
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
            room_id: room_id!("room@prose.org"),
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
            room_id: room_id!("room@prose.org"),
            r#type: RoomEventType::RoomConfigChanged,
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_user_was_permanently_removed() -> Result<()> {
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
        vec![ServerEvent::Room(RoomEvent {
            room_id: room_id!("room@prose.org"),
            r#type: RoomEventType::UserWasPermanentlyRemoved {
                user: RoomUserInfo {
                    id: full!("room@prose.org/nick"),
                    real_id: None,
                    affiliation: RoomAffiliation::None,
                    availability: Availability::Unavailable,
                    is_self: false
                }
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_user_disconnected() -> Result<()> {
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
        vec![ServerEvent::Room(RoomEvent {
            room_id: room_id!("room@prose.org"),
            r#type: RoomEventType::UserAvailabilityOrMembershipChanged {
                user: RoomUserInfo {
                    id: full!("room@prose.org/nick"),
                    real_id: Some(full!("user@prose.org/res")),
                    affiliation: RoomAffiliation::Member,
                    availability: Availability::Unavailable,
                    is_self: false
                }
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_user_was_disconnected_by_server() -> Result<()> {
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
        vec![ServerEvent::Room(RoomEvent {
            room_id: room_id!("room@prose.org"),
            r#type: RoomEventType::UserWasDisconnectedByServer {
                user: RoomUserInfo {
                    id: full!("room@prose.org/nick"),
                    real_id: Some(full!("user@prose.org/res")),
                    affiliation: RoomAffiliation::Member,
                    availability: Availability::Unavailable,
                    is_self: true
                }
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_user_connected() -> Result<()> {
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
        vec![ServerEvent::Room(RoomEvent {
            room_id: room_id!("room@prose.org"),
            r#type: RoomEventType::UserAvailabilityOrMembershipChanged {
                user: RoomUserInfo {
                    id: full!("room@prose.org/nick"),
                    real_id: Some(full!("user@prose.org/res")),
                    affiliation: RoomAffiliation::None,
                    availability: Availability::Available,
                    is_self: false
                }
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
            room_id: room_id!("room@prose.org"),
            r#type: RoomEventType::ReceivedInvite {
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
            room_id: room_id!("room@prose.org"),
            r#type: RoomEventType::ReceivedInvite {
                password: Some("cauldronburn".to_string())
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_compose_state_changed() -> Result<()> {
    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="room@prose.org/user" type="groupchat">
            <composing xmlns="http://jabber.org/protocol/chatstates" />
            <occupant-id xmlns="urn:xmpp:occupant-id:0" id="FvcD+GDkmT8LQAb55uozvL7cZCTBjz3VgQfAcSLtrkM=" />
        </message>
      "#,
    )
        .await?;

    assert_eq!(
        events,
        vec![ServerEvent::Room(RoomEvent {
            room_id: room_id!("room@prose.org"),
            r#type: RoomEventType::UserComposeStateChanged {
                user_id: full!("room@prose.org/user"),
                state: ComposeState::Composing
            },
        })]
    );

    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="user@prose.org/res" type="chat">
            <paused xmlns="http://jabber.org/protocol/chatstates" />
        </message>
      "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::Room(RoomEvent {
            room_id: room_id!("user@prose.org"),
            r#type: RoomEventType::UserComposeStateChanged {
                user_id: full!("user@prose.org/res"),
                state: ComposeState::Idle
            },
        })]
    );

    Ok(())
}
