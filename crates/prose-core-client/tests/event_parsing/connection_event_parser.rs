// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::app::event_handlers::XMPPEvent;
use prose_core_client::domain::shared::models::{ConnectionEvent, ServerEvent};
use prose_core_client::parse_xmpp_event;
use prose_proc_macros::mt_test;
use prose_xmpp::client::Event as XMPPClientEvent;
use prose_xmpp::ConnectionError;

#[mt_test]
async fn test_connected() -> Result<()> {
    let input_event = XMPPEvent::Client(XMPPClientEvent::Connected);
    let output_events = parse_xmpp_event(input_event)?;

    assert_eq!(
        output_events,
        vec![ServerEvent::Connection(ConnectionEvent::Connected)]
    );

    Ok(())
}

#[mt_test]
async fn test_disconnected() -> Result<()> {
    let input_event = XMPPEvent::Client(XMPPClientEvent::Disconnected {
        error: Some(ConnectionError::InvalidCredentials),
    });
    let output_events = parse_xmpp_event(input_event)?;

    assert_eq!(
        output_events,
        vec![ServerEvent::Connection(ConnectionEvent::Disconnected {
            error: Some(ConnectionError::InvalidCredentials)
        })]
    );

    let input_event = XMPPEvent::Client(XMPPClientEvent::Disconnected { error: None });
    let output_events = parse_xmpp_event(input_event)?;

    assert_eq!(
        output_events,
        vec![ServerEvent::Connection(ConnectionEvent::Disconnected { error: None })]
    );

    Ok(())
}
