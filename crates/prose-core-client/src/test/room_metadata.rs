// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::rooms::models::{RoomAffiliation, RoomSessionInfo, RoomSessionMember};
use crate::domain::shared::models::RoomType;
use crate::dtos::{RoomId, UserId};
use crate::test::mock_data;

impl RoomSessionMember {
    pub fn owner(id: UserId, nick: Option<&str>) -> Self {
        Self {
            id,
            affiliation: RoomAffiliation::Owner,
            nick: nick.map(ToString::to_string),
        }
    }

    pub fn member(id: UserId, nick: Option<&str>) -> Self {
        Self {
            id,
            affiliation: RoomAffiliation::Member,
            nick: nick.map(ToString::to_string),
        }
    }

    pub fn admin(id: UserId, nick: Option<&str>) -> Self {
        Self {
            id,
            affiliation: RoomAffiliation::Admin,
            nick: nick.map(ToString::to_string),
        }
    }
}

impl RoomSessionInfo {
    pub fn new_room(room_jid: impl Into<RoomId>, room_type: RoomType) -> Self {
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

    pub fn with_members(mut self, members: impl IntoIterator<Item = RoomSessionMember>) -> Self {
        self.members = members.into_iter().collect();
        self
    }
}
