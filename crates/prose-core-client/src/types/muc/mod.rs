// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub(crate) use bookmark_metadata::{BookmarkMetadata, RoomType};
pub(crate) use room_config::RoomConfig;
pub(crate) use service::{CreateRoomResult, Service};

mod bookmark_metadata;
mod room_config;
mod service;
