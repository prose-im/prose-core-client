// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::{BareJid, Jid};
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::muc::user::Affiliation;

use crate::domain::rooms::models::{RoomInfo, RoomInternals};
use crate::domain::shared::models::RoomType;
use crate::dtos::Occupant;
use crate::test::mock_data;

impl RoomInternals {
    pub fn group(jid: &BareJid) -> Self {
        Self {
            info: RoomInfo {
                jid: jid.clone(),
                name: None,
                description: None,
                user_jid: mock_data::account_jid().into_bare(),
                user_nickname: mock_data::account_jid().node_str().unwrap().to_string(),
                members: vec![],
                room_type: RoomType::Group,
            },
            state: Default::default(),
        }
    }

    pub fn with_occupants(self, occupant: impl IntoIterator<Item = (Jid, Occupant)>) -> Self {
        self.state.write().occupants = occupant.into_iter().collect();
        self
    }
}

impl Occupant {
    pub fn owner() -> Self {
        Occupant {
            jid: None,
            affiliation: Affiliation::Owner,
            chat_state: ChatState::Gone,
            chat_state_updated: Default::default(),
        }
    }

    pub fn set_real_jid(mut self, jid: &BareJid) -> Self {
        self.jid = Some(jid.clone());
        self
    }
}
