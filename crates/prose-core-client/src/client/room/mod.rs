// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod base;
mod channel;
mod direct_message;
mod muc;
mod room;
mod room_envelope;

pub use base::Base;
pub use channel::{PrivateChannel, PublicChannel};
pub use direct_message::DirectMessage;
pub use muc::MUC;
pub use room::{Generic, Group, Room};
pub(super) use room_envelope::RoomEnvelope;

const MESSAGE_PAGE_SIZE: u32 = 50;
