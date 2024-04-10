// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use chrono::{FixedOffset, TimeZone, Utc};
use mockall::predicate;

use prose_core_client::app::event_handlers::{
    RequestEvent, RequestEventType, RequestsEventHandler, ServerEvent, ServerEventHandler,
};
use prose_core_client::domain::general::models::{Capabilities, Feature};
use prose_core_client::domain::shared::models::{CapabilitiesId, RequestId, SenderId};
use prose_core_client::dtos::SoftwareVersion;
use prose_core_client::sender_id;
use prose_core_client::test::{ConstantTimeProvider, MockAppDependencies};
use prose_xmpp::ns;

#[tokio::test]
async fn test_handles_ping() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.request_handling_service
        .expect_respond_to_ping()
        .once()
        .with(
            predicate::eq(sender_id!("sender@prose.org")),
            predicate::eq(RequestId::from("request-id")),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    let event_handler = RequestsEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Request(RequestEvent {
            sender_id: sender_id!("sender@prose.org"),
            request_id: RequestId::from("request-id"),
            r#type: RequestEventType::Ping,
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
            predicate::eq(sender_id!("sender@prose.org")),
            predicate::eq(RequestId::from("my-request")),
            predicate::eq(
                Utc.with_ymd_and_hms(2023, 09, 10, 0, 0, 0)
                    .unwrap()
                    .with_timezone(&FixedOffset::east_opt(0).unwrap()),
            ),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    let event_handler = RequestsEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Request(RequestEvent {
            sender_id: sender_id!("sender@prose.org"),
            request_id: RequestId::from("my-request"),
            r#type: RequestEventType::LocalTime,
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
            predicate::eq(sender_id!("sender@prose.org")),
            predicate::eq(RequestId::from("my-request")),
            predicate::eq(SoftwareVersion {
                name: "my-client".to_string(),
                version: "3000".to_string(),
                os: None,
            }),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    let event_handler = RequestsEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Request(RequestEvent {
            sender_id: sender_id!("sender@prose.org"),
            request_id: RequestId::from("my-request"),
            r#type: RequestEventType::SoftwareVersion,
        }))
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
            predicate::eq(sender_id!("sender@prose.org")),
            predicate::eq(RequestId::from("my-request")),
            predicate::eq(0),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    let event_handler = RequestsEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Request(RequestEvent {
            sender_id: sender_id!("sender@prose.org"),
            request_id: RequestId::from("my-request"),
            r#type: RequestEventType::LastActivity,
        }))
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
            predicate::eq(sender_id!("sender@prose.org")),
            predicate::eq(RequestId::from("my-request")),
            predicate::eq(Capabilities::new(
                "My Client",
                "https://example.com",
                vec![Feature::Name(ns::ROSTER)],
            )),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    let event_handler = RequestsEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(ServerEvent::Request(RequestEvent {
            sender_id: sender_id!("sender@prose.org"),
            request_id: RequestId::from("my-request"),
            r#type: RequestEventType::Capabilities {
                id: CapabilitiesId::from("caps-id"),
            },
        }))
        .await?;

    Ok(())
}
