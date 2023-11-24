// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;

use crate::domain::rooms::models::RoomSessionInfo;
use crate::domain::shared::models::RoomType;
use crate::dtos::RoomJid;
use crate::test::mock_data;

impl RoomSessionInfo {
    pub fn new_room(room_jid: impl Into<RoomJid>, room_type: RoomType) -> Self {
        Self {
            room_jid: room_jid.into(),
            room_name: None,
            room_description: None,
            room_type,
            user_nickname: mock_data::account_jid().node_str().unwrap().to_string(),
            members: vec![],
            room_has_been_created: true,
        }
    }

    pub fn with_members(mut self, members: impl IntoIterator<Item = BareJid>) -> Self {
        self.members = members.into_iter().collect();
        self
    }
}
