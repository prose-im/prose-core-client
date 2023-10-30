// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use chrono::{TimeZone, Utc};
use mockall::predicate;

use prose_core_client::app::event_handlers::{RequestsEventHandler, XMPPEvent, XMPPEventHandler};
use prose_core_client::domain::general::models::{Capabilities, Feature};
use prose_core_client::domain::general::services::SubscriptionResponse;
use prose_core_client::dtos::SoftwareVersion;
use prose_core_client::test::{ConstantTimeProvider, MockAppDependencies};
use prose_xmpp::{bare, jid, mods, ns};

#[tokio::test]
async fn test_handles_ping() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.request_handling_service
        .expect_respond_to_ping()
        .once()
        .with(
            predicate::eq(jid!("sender@prose.org")),
            predicate::eq("request-id"),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    let event_handler = RequestsEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(XMPPEvent::Ping(mods::ping::Event::Ping {
            from: jid!("sender@prose.org"),
            id: "request-id".to_string(),
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_entity_time_query() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    deps.time_provider = Arc::new(ConstantTimeProvider::ymd(2023, 09, 10));

    deps.request_handling_service
        .expect_respond_to_entity_time_request()
        .once()
        .with(
            predicate::eq(jid!("sender@prose.org")),
            predicate::eq("my-request"),
            predicate::eq(Utc.with_ymd_and_hms(2023, 09, 10, 0, 0, 0).unwrap()),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    let event_handler = RequestsEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(XMPPEvent::Profile(mods::profile::Event::EntityTimeQuery {
            from: jid!("sender@prose.org"),
            id: "my-request".to_string(),
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_software_version_query() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    deps.ctx.software_version = SoftwareVersion {
        name: "my-client".to_string(),
        version: "3000".to_string(),
        os: None,
    };

    deps.request_handling_service
        .expect_respond_to_software_version_request()
        .once()
        .with(
            predicate::eq(jid!("sender@prose.org")),
            predicate::eq("my-request"),
            predicate::eq(SoftwareVersion {
                name: "my-client".to_string(),
                version: "3000".to_string(),
                os: None,
            }),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    let event_handler = RequestsEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(XMPPEvent::Profile(
            mods::profile::Event::SoftwareVersionQuery {
                from: jid!("sender@prose.org"),
                id: "my-request".to_string(),
            },
        ))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_last_activity_request() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    deps.ctx.software_version = SoftwareVersion {
        name: "my-client".to_string(),
        version: "3000".to_string(),
        os: None,
    };

    deps.request_handling_service
        .expect_respond_to_last_activity_request()
        .once()
        .with(
            predicate::eq(jid!("sender@prose.org")),
            predicate::eq("my-request"),
            predicate::eq(0),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    let event_handler = RequestsEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(XMPPEvent::Profile(
            mods::profile::Event::LastActivityQuery {
                from: jid!("sender@prose.org"),
                id: "my-request".to_string(),
            },
        ))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_disco_request() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    deps.ctx.capabilities = Capabilities::new(
        "My Client",
        "https://example.com",
        vec![Feature::Name(ns::ROSTER)],
    );

    deps.request_handling_service
        .expect_respond_to_disco_info_query()
        .once()
        .with(
            predicate::eq(jid!("sender@prose.org")),
            predicate::eq("my-request"),
            predicate::eq(Capabilities::new(
                "My Client",
                "https://example.com",
                vec![Feature::Name(ns::ROSTER)],
            )),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    let event_handler = RequestsEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(XMPPEvent::Caps(mods::caps::Event::DiscoInfoQuery {
            from: jid!("sender@prose.org"),
            id: "my-request".to_string(),
            node: None,
        }))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_presence_subscription_request() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    deps.ctx.capabilities = Capabilities::new(
        "My Client",
        "https://example.com",
        vec![Feature::Name(ns::ROSTER)],
    );

    deps.request_handling_service
        .expect_respond_to_presence_subscription_request()
        .once()
        .with(
            predicate::eq(bare!("sender@prose.org")),
            predicate::eq(SubscriptionResponse::Approve),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    let event_handler = RequestsEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(XMPPEvent::Roster(
            mods::roster::Event::PresenceSubscriptionRequest {
                from: bare!("sender@prose.org"),
            },
        ))
        .await?;

    Ok(())
}
