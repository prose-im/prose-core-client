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
