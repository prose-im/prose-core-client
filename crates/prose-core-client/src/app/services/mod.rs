// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use account_service::AccountService;
pub use cache_service::CacheService;
pub use connection_service::ConnectionService;
pub use contacts_service::ContactsService;
pub(crate) use room::RoomInner;
pub use room::{DirectMessage, Generic, Group, PrivateChannel, PublicChannel, Room};
pub use room_envelope::RoomEnvelope;
pub use rooms_service::RoomsService;
pub use sidebar_service::SidebarService;
pub use user_data_service::UserDataService;

mod account_service;
mod cache_service;
mod connection_service;
mod contacts_service;
mod room;
mod room_envelope;
mod rooms_service;
mod sidebar_service;
mod user_data_service;
