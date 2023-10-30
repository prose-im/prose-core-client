// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

pub(crate) use client_event_dispatcher::ClientEventDispatcher;
pub(crate) use connection_event_handler::ConnectionEventHandler;
pub(crate) use event_handler_queue::XMPPEventHandlerQueue;
pub(crate) use messages_event_handler::MessagesEventHandler;
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
pub(crate) use prose_xmpp::Event as XMPPEvent;
pub(crate) use requests_event_handler::RequestsEventHandler;
pub(crate) use rooms_event_handler::RoomsEventHandler;
pub(crate) use user_state_event_handler::UserStateEventHandler;

mod client_event_dispatcher;
mod connection_event_handler;
mod event_handler_queue;
mod messages_event_handler;
mod requests_event_handler;
mod rooms_event_handler;
mod user_state_event_handler;

/// `XMPPEventHandler` is a trait representing a handler for XMPP events.
///
/// Implementors of this trait should provide a `handle_event` method, which takes an `XMPPEvent`
/// and returns an `Option<XMPPEvent>`. If the handler returns `None`, it means the event has been
/// consumed and no further processing should be done. If it returns `Some(event)`, the event is
/// not consumed and should be passed to the next handler.
#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
pub(crate) trait XMPPEventHandler: SendUnlessWasm + SyncUnlessWasm {
    fn name(&self) -> &'static str;
    async fn handle_event(&self, event: XMPPEvent) -> Result<Option<XMPPEvent>>;
}

#[cfg_attr(feature = "test", mockall::automock)]
pub trait EventDispatcher<E>: SendUnlessWasm + SyncUnlessWasm
where
    E: SendUnlessWasm + SyncUnlessWasm,
{
    fn dispatch_event(&self, event: E);
}
