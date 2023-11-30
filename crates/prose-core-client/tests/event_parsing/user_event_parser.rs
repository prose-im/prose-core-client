// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::domain::shared::models::{
    CapabilitiesId, ServerEvent, UserResourceEvent, UserResourceEventType, UserStatusEvent,
    UserStatusEventType,
};
use prose_core_client::dtos::*;
use prose_core_client::test::parse_xml;
use prose_core_client::{occupant_id, user_id, user_resource_id};
use prose_proc_macros::mt_test;

#[mt_test]
async fn test_user_presence_and_capabilities() -> Result<()> {
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
                    id: CapabilitiesId::from(
                        "https://prose.org#ImujI7nqf7pn4YqcjefXE3o5P1k=".to_string()
                    )
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
