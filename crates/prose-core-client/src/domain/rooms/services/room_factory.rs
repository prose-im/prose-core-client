// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use crate::app::services::{RoomEnvelope, RoomInner};
use crate::domain::rooms::models::RoomInternals;
use crate::domain::shared::models::RoomType;

#[derive(Clone)]
pub struct RoomFactory {
    builder: Arc<dyn Fn(Arc<RoomInternals>) -> RoomInner + Send + Sync>,
}

impl RoomFactory {
    pub fn new<F>(builder: F) -> Self
    where
        F: Fn(Arc<RoomInternals>) -> RoomInner + Send + Sync + 'static,
    {
        RoomFactory {
            builder: Arc::new(builder),
        }
    }

    pub fn build(&self, room: Arc<RoomInternals>) -> RoomEnvelope {
        let room_type = room.info.room_type.clone();
        let inner = Arc::new((self.builder)(room));

        match room_type {
            RoomType::Pending => panic!("Cannot convert pending room to RoomEnvelope"),
            RoomType::DirectMessage => RoomEnvelope::DirectMessage(inner.into()),
            RoomType::Group => RoomEnvelope::Group(inner.into()),
            RoomType::PrivateChannel => RoomEnvelope::PrivateChannel(inner.into()),
            RoomType::PublicChannel => RoomEnvelope::PublicChannel(inner.into()),
            RoomType::Generic => RoomEnvelope::Generic(inner.into()),
        }
    }
}
