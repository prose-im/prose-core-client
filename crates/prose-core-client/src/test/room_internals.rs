// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use jid::{BareJid, Jid};
use std::collections::HashMap;
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::muc::user::Affiliation;

use crate::domain::rooms::models::{RoomInfo, RoomInternals};
use crate::domain::shared::models::{RoomJid, RoomType};
use crate::dtos::{Member, Occupant};
use crate::test::mock_data;

impl RoomInternals {
    pub fn group(jid: impl Into<RoomJid>) -> Self {
        Self {
            info: RoomInfo {
                jid: jid.into(),
                description: None,
                user_jid: mock_data::account_jid().into_bare(),
                user_nickname: mock_data::account_jid().node_str().unwrap().to_string(),
                members: HashMap::new(),
                room_type: RoomType::Group,
            },
            state: Default::default(),
        }
    }

    pub fn public_channel(jid: impl Into<RoomJid>) -> Self {
        Self {
            info: RoomInfo {
                jid: jid.into(),
                description: None,
                user_jid: mock_data::account_jid().into_bare(),
                user_nickname: mock_data::account_jid().node_str().unwrap().to_string(),
                members: HashMap::new(),
                room_type: RoomType::PublicChannel,
            },
            state: Default::default(),
        }
    }

    pub fn private_channel(jid: impl Into<RoomJid>) -> Self {
        Self {
            info: RoomInfo {
                jid: jid.into(),
                description: None,
                user_jid: mock_data::account_jid().into_bare(),
                user_nickname: mock_data::account_jid().node_str().unwrap().to_string(),
                members: HashMap::new(),
                room_type: RoomType::PrivateChannel,
            },
            state: Default::default(),
        }
    }

    pub fn with_user_nickname(mut self, nickname: impl Into<String>) -> Self {
        self.info.user_nickname = nickname.into();
        self
    }

    pub fn with_name(self, name: impl Into<String>) -> Self {
        self.state.write().name = Some(name.into());
        self
    }

    pub fn with_members(mut self, members: impl IntoIterator<Item = (BareJid, Member)>) -> Self {
        self.info.members = members.into_iter().collect();
        self
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
            name: None,
            affiliation: Affiliation::Owner,
            chat_state: ChatState::Gone,
            chat_state_updated: Default::default(),
        }
    }

    pub fn member() -> Self {
        Occupant {
            jid: None,
            name: None,
            affiliation: Affiliation::Owner,
            chat_state: ChatState::Gone,
            chat_state_updated: Default::default(),
        }
    }

    pub fn set_real_jid(mut self, jid: &BareJid) -> Self {
        self.jid = Some(jid.clone());
        self
    }

    pub fn set_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn set_chat_state(mut self, chat_state: ChatState) -> Self {
        self.chat_state = chat_state;
        self
    }

    pub fn set_chat_state_updated(mut self, timestamp: DateTime<Utc>) -> Self {
        self.chat_state_updated = timestamp;
        self
    }
}
