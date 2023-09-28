// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

mod channel;
mod direct_message;
mod generic;
mod group;
mod muc;
mod room;
mod room_envelope;

pub use channel::{PrivateChannel, PublicChannel};
pub use direct_message::DirectMessage;
pub use generic::Generic;
pub use group::Group;
pub use muc::MUC;
pub use room::Room;
pub(super) use room_envelope::RoomEnvelope;

#[cfg(feature = "debug")]
pub use room::Occupant;

const MESSAGE_PAGE_SIZE: u32 = 50;
