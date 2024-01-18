// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use crate::app::services::{RoomEnvelope, RoomInner};
use crate::domain::rooms::models::Room;
use crate::domain::shared::models::RoomType;

#[cfg(target_arch = "wasm32")]
pub type RoomBuilder = Arc<dyn Fn(Room) -> RoomInner>;
#[cfg(not(target_arch = "wasm32"))]
pub type RoomBuilder = Arc<dyn Fn(Room) -> RoomInner + Send + Sync>;

#[derive(Clone)]
pub struct RoomFactory {
    builder: RoomBuilder,
}

impl RoomFactory {
    pub fn new(builder: RoomBuilder) -> Self {
        RoomFactory { builder }
    }

    pub fn build(&self, room: Room) -> RoomEnvelope {
        let room_type = room.r#type.clone();
        let inner = Arc::new((self.builder)(room));

        match room_type {
            RoomType::Unknown => {
                panic!("Cannot convert pending room to RoomEnvelope")
            }
            RoomType::DirectMessage => RoomEnvelope::DirectMessage(inner.into()),
            RoomType::Group => RoomEnvelope::Group(inner.into()),
            RoomType::PrivateChannel => RoomEnvelope::PrivateChannel(inner.into()),
            RoomType::PublicChannel => RoomEnvelope::PublicChannel(inner.into()),
            RoomType::Generic => RoomEnvelope::Generic(inner.into()),
        }
    }
}
