// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use account_service::AccountService;
pub use contacts_service::ContactsService;
pub(crate) use room::RoomInner;
pub use room::{DirectMessage, Generic, Group, PrivateChannel, PublicChannel, Room};
pub use room_envelope::RoomEnvelope;
pub use rooms_service::RoomsService;
pub(crate) use rooms_service::{CreateOrEnterRoomRequest, CreateOrEnterRoomRequestType};
pub use user_data_service::UserDataService;

mod account_service;
mod contacts_service;
mod room;
mod room_envelope;
mod rooms_service;
mod user_data_service;
