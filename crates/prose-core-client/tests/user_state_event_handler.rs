// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use mockall::predicate;
use xmpp_parsers::presence::Presence;

use prose_core_client::app::event_handlers::{
    ServerEventHandler, UserStateEventHandler, XMPPEvent, XMPPEventHandler,
};
use prose_core_client::domain::connection::models::ConnectionProperties;
use prose_core_client::domain::shared::models::{Availability, UserId, UserResourceId};
use prose_core_client::domain::user_info::models::Presence as DomainPresence;
use prose_core_client::test::MockAppDependencies;
use prose_core_client::{user_id, user_resource_id, ClientEvent};
use prose_xmpp::{bare, full, jid, mods};

#[tokio::test]
async fn test_handles_presence() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.user_info_repo
        .expect_set_user_presence()
        .once()
        .with(
            predicate::eq(user_resource_id!("sender@prose.org/resource")),
            predicate::eq(DomainPresence {
                priority: 1,
                availability: Availability::Available,
                status: None,
            }),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::ContactChanged {
            id: user_id!("sender@prose.org"),
        }))
        .return_once(|_| ());

    let event_handler = UserStateEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(XMPPEvent::Status(mods::status::Event::Presence(
            Presence::available()
                .with_from(jid!("sender@prose.org/resource"))
                .with_priority(1),
        )))
        .await?;

    Ok(())
}

#[tokio::test]
/// Test that UserStateEventHandler does not send an event when a self-presence is received and
/// that the event is consumed, i.e. cannot be forwarded to other handlers.
async fn test_swallows_self_presence() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.ctx.set_connection_properties(ConnectionProperties {
        connected_jid: full!("hello@prose.org/res"),
        server_features: Default::default(),
    });

    deps.user_info_repo
        .expect_set_user_presence()
        .once()
        .with(
            predicate::eq(jid!("hello@prose.org")),
            predicate::eq(DomainPresence {
                availability: Availability::Available,
                ..Default::default()
            }),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    let event_handler = UserStateEventHandler::from(&deps.into_deps());
    assert!(event_handler
        .handle_event(XMPPEvent::Status(
            mods::status::Event::Presence(Presence::available().with_from(jid!("hello@prose.org")),)
        ))
        .await?
        .is_none());

    Ok(())
}
