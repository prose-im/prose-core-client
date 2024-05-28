// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

pub use block_list_event_handler::BlockListEventHandler;
pub use bookmarks_event_handler::BookmarksEventHandler;
pub use connection_event_handler::ConnectionEventHandler;
pub use contact_list_event_handler::ContactListEventHandler;
pub use event_handler_queue::ServerEventHandlerQueue;
pub use messages_event_handler::MessagesEventHandler;
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
pub use prose_xmpp::Event as XMPPEvent;
pub use requests_event_handler::RequestsEventHandler;
pub use rooms_event_handler::RoomsEventHandler;
pub use server_event::*;
pub use synced_room_settings_event_handler::SyncedRoomSettingsEventHandler;
pub use user_devices_event_handler::UserDevicesEventHandler;
pub use user_state_event_handler::UserStateEventHandler;

use crate::domain::rooms::models::Room;
use crate::{ClientEvent, ClientRoomEventType};

mod block_list_event_handler;
mod bookmarks_event_handler;
mod connection_event_handler;
mod contact_list_event_handler;
mod event_handler_queue;
mod messages_event_handler;
mod requests_event_handler;
mod rooms_event_handler;
mod server_event;
mod synced_room_settings_event_handler;
mod user_devices_event_handler;
mod user_state_event_handler;

/// `ServerEventHandler` is a trait representing a handler for XMPP events.
///
/// Implementors of this trait should provide a `handle_event` method, which takes an `XMPPEvent`
/// and returns an `Option<ServerEvent>`. If the handler returns `None`, it means the event has been
/// consumed and no further processing should be done. If it returns `Some(event)`, the event is
/// not consumed and should be passed to the next handler.
#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
pub trait ServerEventHandler: SendUnlessWasm + SyncUnlessWasm {
    fn name(&self) -> &'static str;
    async fn handle_event(&self, event: ServerEvent) -> Result<Option<ServerEvent>>;
}

#[cfg_attr(feature = "test", mockall::automock)]
pub trait ClientEventDispatcherTrait: SendUnlessWasm + SyncUnlessWasm {
    fn dispatch_event(&self, event: ClientEvent);
    fn dispatch_room_event(&self, room: Room, event: ClientRoomEventType);
}
