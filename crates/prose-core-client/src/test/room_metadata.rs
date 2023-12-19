// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::rooms::models::{
    RoomAffiliation, RoomConfig, RoomSessionInfo, RoomSessionMember,
};
use crate::domain::shared::models::RoomType;
use crate::dtos::{RoomId, UserId};
use crate::test::mock_data;

impl RoomSessionMember {
    pub fn owner(id: UserId) -> Self {
        Self {
            id,
            affiliation: RoomAffiliation::Owner,
        }
    }

    pub fn member(id: UserId) -> Self {
        Self {
            id,
            affiliation: RoomAffiliation::Member,
        }
    }

    pub fn admin(id: UserId) -> Self {
        Self {
            id,
            affiliation: RoomAffiliation::Admin,
        }
    }
}

impl RoomSessionInfo {
    pub fn new_room(room_jid: impl Into<RoomId>, room_type: RoomType) -> Self {
        Self {
            room_id: room_jid.into(),
            config: RoomConfig {
                room_name: None,
                room_description: None,
                room_type,
            },
            user_nickname: mock_data::account_jid().username().to_string(),
            members: vec![],
            room_has_been_created: true,
        }
    }

    pub fn with_members(mut self, members: impl IntoIterator<Item = RoomSessionMember>) -> Self {
        self.members = members.into_iter().collect();
        self
    }
}
