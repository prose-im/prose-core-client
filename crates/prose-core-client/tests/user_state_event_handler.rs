// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use mockall::predicate;
use xmpp_parsers::presence::Presence;

use prose_core_client::app::event_handlers::{UserStateEventHandler, XMPPEvent, XMPPEventHandler};
use prose_core_client::domain::user_info::models::Presence as DomainPresence;
use prose_core_client::dtos::Availability;
use prose_core_client::test::MockAppDependencies;
use prose_core_client::ClientEvent;
use prose_xmpp::{bare, jid, mods};

#[tokio::test]
async fn test_handles_presence() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.user_info_repo
        .expect_set_user_presence()
        .once()
        .with(
            predicate::eq(jid!("sender@prose.org/resource")),
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
            jid: bare!("sender@prose.org"),
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
