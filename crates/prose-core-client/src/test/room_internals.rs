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
use crate::util::jid_ext::BareJidExt;

impl RoomInternals {
    pub fn direct_message(jid: impl Into<BareJid>) -> Self {
        let jid = jid.into();

        Self::for_direct_message(
            &jid,
            &mock_data::account_jid().into_bare(),
            &jid.to_display_name(),
        )
    }

    pub fn mock_pending_room(jid: impl Into<RoomJid>, next_hash: &str) -> Self {
        Self::pending(
            &jid.into(),
            &mock_data::account_jid().into_bare(),
            &format!(
                "{}-{}",
                mock_data::account_jid().node_str().unwrap(),
                next_hash
            ),
        )
    }

    pub fn group(jid: impl Into<RoomJid>) -> Self {
        Self::new(RoomInfo {
            jid: jid.into(),
            description: None,
            user_jid: mock_data::account_jid().into_bare(),
            user_nickname: mock_data::account_jid().node_str().unwrap().to_string(),
            members: HashMap::new(),
            r#type: RoomType::Group,
        })
    }

    pub fn public_channel(jid: impl Into<RoomJid>) -> Self {
        Self::new(RoomInfo {
            jid: jid.into(),
            description: None,
            user_jid: mock_data::account_jid().into_bare(),
            user_nickname: mock_data::account_jid().node_str().unwrap().to_string(),
            members: HashMap::new(),
            r#type: RoomType::PublicChannel,
        })
    }

    pub fn private_channel(jid: impl Into<RoomJid>) -> Self {
        Self::new(RoomInfo {
            jid: jid.into(),
            description: None,
            user_jid: mock_data::account_jid().into_bare(),
            user_nickname: mock_data::account_jid().node_str().unwrap().to_string(),
            members: HashMap::new(),
            r#type: RoomType::PrivateChannel,
        })
    }

    pub fn with_user_nickname(mut self, nickname: impl Into<String>) -> Self {
        self.user_nickname = nickname.into();
        self
    }

    pub fn with_name(self, name: impl AsRef<str>) -> Self {
        self.set_name(name.as_ref());
        self
    }

    pub fn with_members(mut self, members: impl IntoIterator<Item = (BareJid, Member)>) -> Self {
        self.members = members.into_iter().collect();
        self
    }

    pub fn with_occupants(self, occupant: impl IntoIterator<Item = (Jid, Occupant)>) -> Self {
        self.set_occupants(occupant.into_iter().collect());
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
