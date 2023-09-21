// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::FullJid;
use xmpp_parsers::muc::user::Status;

pub(crate) use bookmark_metadata::{BookmarkMetadata, RoomType};
use prose_xmpp::mods::muc::RoomOccupancy;
pub(crate) use room_config::RoomConfig;
pub(crate) use room_settings::{RoomSettings, RoomValidationError};
pub(crate) use service::Service;

mod bookmark_metadata;
mod room_config;
mod room_settings;
mod service;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct RoomMetadata {
    pub room_jid: FullJid,
    pub occupancy: RoomOccupancy,
    pub settings: RoomSettings,
}

impl RoomMetadata {
    pub fn room_has_been_created(&self) -> bool {
        self.occupancy
            .user
            .status
            .contains(&Status::RoomHasBeenCreated)
    }
}