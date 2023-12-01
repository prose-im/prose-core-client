// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::app::event_handlers::{RequestEvent, RequestEventType, ServerEvent};
use prose_core_client::domain::shared::models::{CapabilitiesId, RequestId, SenderId};
use prose_core_client::sender_id;
use prose_core_client::test::parse_xml;
use prose_proc_macros::mt_test;

#[mt_test]
async fn test_ping() -> Result<()> {
    // XEP-0199: XMPP Ping
    // https://xmpp.org/extensions/xep-0199.html
    let events =
        parse_xml(
            r#"
        <iq xmlns="jabber:client" from="prose.org" id="req-id" type="get">
          <ping xmlns='urn:xmpp:ping'/>
        </iq>
        "#,
        )
        .await?;

    assert_eq!(
        events,
        vec![ServerEvent::Request(RequestEvent {
            request_id: RequestId::from("req-id"),
            sender_id: sender_id!("prose.org"),
            r#type: RequestEventType::Ping,
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_local_time() -> Result<()> {
    // XEP-0202: Entity Time
    // https://xmpp.org/extensions/xep-0202.html

    let events =
        parse_xml(
            r#"
        <iq xmlns="jabber:client" from="user@prose.org/res" id="req-id" type="get">
          <time xmlns="urn:xmpp:time" />
        </iq>
        "#,
        )
        .await?;

    assert_eq!(
        events,
        vec![ServerEvent::Request(RequestEvent {
            request_id: RequestId::from("req-id"),
            sender_id: sender_id!("user@prose.org/res"),
            r#type: RequestEventType::LocalTime,
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_last_activity() -> Result<()> {
    // XEP-0012: Last Activity
    // https://xmpp.org/extensions/xep-0012.html

    let events = parse_xml(
        r#"
        <iq xmlns="jabber:client" from="user@prose.org/res" id="req-id" type="get">
          <query xmlns="jabber:iq:last" />
        </iq>
        "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::Request(RequestEvent {
            request_id: RequestId::from("req-id"),
            sender_id: sender_id!("user@prose.org/res"),
            r#type: RequestEventType::LastActivity,
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_capabilities() -> Result<()> {
    // XEP-0115: Entity Capabilities
    // https://xmpp.org/extensions/xep-0115.html

    let events = parse_xml(
        r#"
        <iq xmlns="jabber:client" from="user@prose.org/res" id="req-id" type="get">
          <query xmlns="http://jabber.org/protocol/disco#info" node="https://prose.org#ImujI7nqf7pn4YqcjefXE3o5P1k=" />
        </iq>
        "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::Request(RequestEvent {
            request_id: RequestId::from("req-id"),
            sender_id: sender_id!("user@prose.org/res"),
            r#type: RequestEventType::Capabilities {
                id: CapabilitiesId::from("https://prose.org#ImujI7nqf7pn4YqcjefXE3o5P1k=")
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_software_version() -> Result<()> {
    // XEP-0092: Software Version
    // https://xmpp.org/extensions/xep-0092.html

    let events = parse_xml(
        r#"
        <iq xmlns="jabber:client" from="user@prose.org/res" id="req-id" type="get">
          <query xmlns="jabber:iq:version" />
        </iq>
        "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::Request(RequestEvent {
            request_id: RequestId::from("req-id"),
            sender_id: sender_id!("user@prose.org/res"),
            r#type: RequestEventType::SoftwareVersion,
        })]
    );

    Ok(())
}
