// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use account_service::AccountService;
pub use block_list_service::BlockListService;
pub use cache_service::CacheService;
pub use connection_service::ConnectionService;
pub use contact_list_service::ContactListService;
#[cfg(feature = "debug")]
pub use debug_service::DebugService;
pub(crate) use room::RoomInner;
pub use room::{DirectMessage, Generic, Group, PrivateChannel, PublicChannel, Room};
pub use rooms_service::RoomsService;
pub use sidebar_service::SidebarService;
pub use upload_service::UploadService;
pub use user_data_service::UserDataService;

mod account_service;
mod block_list_service;
mod cache_service;
mod connection_service;
mod contact_list_service;
#[cfg(feature = "debug")]
mod debug_service;
pub(crate) mod room;
mod rooms_service;
mod sidebar_service;
mod upload_service;
mod user_data_service;
