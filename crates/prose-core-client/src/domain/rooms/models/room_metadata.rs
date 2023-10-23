// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::{BareJid, FullJid};
use xmpp_parsers::muc::user::Status;

use prose_xmpp::mods::muc::RoomOccupancy;

use crate::domain::rooms::models::RoomSettings;

#[derive(Debug, PartialEq, Clone)]
pub struct RoomMetadata {
    pub room_jid: FullJid,
    pub occupancy: RoomOccupancy,
    pub settings: RoomSettings,
    pub members: Vec<BareJid>,
}

impl RoomMetadata {
    pub fn room_has_been_created(&self) -> bool {
        self.occupancy
            .user
            .status
            .contains(&Status::RoomHasBeenCreated)
    }
}
