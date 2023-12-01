// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::domain::rooms::models::ComposeState;
use prose_core_client::domain::shared::models::{
    CapabilitiesId, ServerEvent, UserInfoEvent, UserInfoEventType, UserResourceEvent,
    UserResourceEventType, UserStatusEvent, UserStatusEventType,
};
use prose_core_client::domain::user_info::models::{AvatarImageId, AvatarMetadata};
use prose_core_client::dtos::*;
use prose_core_client::test::parse_xml;
use prose_core_client::{occupant_id, user_id, user_resource_id};
use prose_proc_macros::mt_test;

#[mt_test]
async fn test_user_presence_and_capabilities_changed() -> Result<()> {
    // Initial unavailable presence
    let events = parse_xml(
        r#"
        <presence xmlns="jabber:client" from="user@prose.org" type="unavailable" />
        "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::UserStatus(UserStatusEvent {
            user_id: user_id!("user@prose.org").into(),
            r#type: UserStatusEventType::AvailabilityChanged {
                availability: Availability::Unavailable
            },
        })]
    );

    // User comes online
    // https://xmpp.org/extensions/xep-0115.html
    let events = parse_xml(
    r#"
        <presence xmlns="jabber:client" from="user@prose.org/res">
          <show>chat</show>
          <c xmlns="http://jabber.org/protocol/caps" hash="sha-1" node="https://prose.org" ver="ImujI7nqf7pn4YqcjefXE3o5P1k=" />
          <x xmlns="vcard-temp:x:update">
            <photo>cdc05cb9c48d5e817a36d462fe0470a0579e570a</photo>
          </x>
          <delay xmlns="urn:xmpp:delay" from="prose.org" stamp="2023-11-30T20:11:37Z" />
        </presence>
        "#,
  )
      .await?;

    assert_eq!(
        events,
        vec![
            ServerEvent::UserStatus(UserStatusEvent {
                user_id: user_resource_id!("user@prose.org/res").into(),
                r#type: UserStatusEventType::AvailabilityChanged {
                    availability: Availability::Available
                },
            }),
            ServerEvent::UserResource(UserResourceEvent {
                user_id: user_resource_id!("user@prose.org/res"),
                r#type: UserResourceEventType::CapabilitiesChanged {
                    id: CapabilitiesId::from("https://prose.org#ImujI7nqf7pn4YqcjefXE3o5P1k=")
                },
            })
        ]
    );

    // Exiting a Room (https://xmpp.org/extensions/xep-0045.html#exit)
    let events =
        parse_xml(
            r#"
        <presence xmlns="jabber:client" from="room@prose.org/nick" type="unavailable">
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
                availability: Availability::Unavailable
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_compose_state_changed() -> Result<()> {
    // XEP-0085: Chat State Notifications (https://xmpp.org/extensions/xep-0085.html#top)
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
        vec![ServerEvent::UserStatus(UserStatusEvent {
            user_id: occupant_id!("room@prose.org/user").into(),
            r#type: UserStatusEventType::ComposeStateChanged {
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
        vec![ServerEvent::UserStatus(UserStatusEvent {
            user_id: user_resource_id!("user@prose.org/res").into(),
            r#type: UserStatusEventType::ComposeStateChanged {
                state: ComposeState::Idle
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_avatar_changed() -> Result<()> {
    // XEP-0084: User Avatar
    // https://xmpp.org/extensions/xep-0084.html

    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="user@prose.org" type="headline">
            <event xmlns="http://jabber.org/protocol/pubsub#event">
                <items node="urn:xmpp:avatar:metadata">
                    <item id="fa3c5706e27f6a0093981bb315015c2bd93e094e" publisher="user@prose.org">
                        <metadata xmlns="urn:xmpp:avatar:metadata">
                            <info bytes="61501" id="fa3c5706e27f6a0093981bb315015c2bd93e094e" type="image/jpeg" />
                        </metadata>
                    </item>
                </items>
            </event>
        </message>
      "#,
    )
        .await?;

    assert_eq!(
        events,
        vec![ServerEvent::UserInfo(UserInfoEvent {
            user_id: user_id!("user@prose.org").into(),
            r#type: UserInfoEventType::AvatarChanged {
                metadata: AvatarMetadata {
                    bytes: 61501,
                    mime_type: "image/jpeg".to_string(),
                    checksum: AvatarImageId::from("fa3c5706e27f6a0093981bb315015c2bd93e094e"),
                    width: None,
                    height: None,
                    url: None,
                },
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_profile_changed() -> Result<()> {
    // XEP-0292: vCard4 Over XMPP
    // https://xmpp.org/extensions/xep-0292.html

    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="user@prose.org" type="headline">
            <event xmlns="http://jabber.org/protocol/pubsub#event">
                <items node="urn:ietf:params:xml:ns:vcard-4.0">
                    <item id="user@prose.org" publisher="user@prose.org">
                        <vcard xmlns="urn:ietf:params:xml:ns:vcard-4.0">
                            <adr>
                                <country>DE</country>
                                <locality>Berlin</locality>
                            </adr>
                            <email>
                                <text>user@prose.org</text>
                            </email>
                            <n>
                                <given>Jane</given>
                                <surname>Doe</surname>
                            </n>
                            <org>
                                <text>Prose Foundation</text>
                            </org>
                            <title>
                                <text>Developer</text>
                            </title>
                        </vcard>
                    </item>
                </items>
            </event>
        </message>
      "#,
    )
    .await?;

    let mut profile = UserProfile::default();
    profile.address = Some(Address {
        locality: Some("Berlin".to_string()),
        country: Some("DE".to_string()),
    });
    profile.email = Some("user@prose.org".to_string());
    profile.first_name = Some("Jane".to_string());
    profile.last_name = Some("Doe".to_string());
    profile.org = Some("Prose Foundation".to_string());
    profile.title = Some("Developer".to_string());

    assert_eq!(
        events,
        vec![ServerEvent::UserInfo(UserInfoEvent {
            user_id: user_id!("user@prose.org").into(),
            r#type: UserInfoEventType::ProfileChanged { profile },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_status_changed() -> Result<()> {
    // XEP-0108: User Activity
    // https://xmpp.org/extensions/xep-0108.html

    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="user@prose.org" type="headline">
            <event xmlns="http://jabber.org/protocol/pubsub#event">
                <items node="http://jabber.org/protocol/activity">
                    <item id="user@prose.org" publisher="user@prose.org">
                        <activity xmlns="http://jabber.org/protocol/activity">
                            <undefined>
                                <other>🍕</other>
                            </undefined>
                            <text>Eating pizza</text>
                        </activity>
                    </item>
                </items>
            </event>
        </message>
      "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::UserInfo(UserInfoEvent {
            user_id: user_id!("user@prose.org").into(),
            r#type: UserInfoEventType::StatusChanged {
                status: UserStatus {
                    emoji: "🍕".to_string(),
                    status: Some("Eating pizza".to_string())
                }
            },
        })]
    );

    Ok(())
}
