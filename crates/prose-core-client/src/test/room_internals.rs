// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use jid::{BareJid, Jid};

use crate::domain::rooms::models::{ComposeState, RoomAffiliation, RoomInfo, RoomInternals};
use crate::domain::shared::models::{RoomId, RoomType};
use crate::dtos::{Availability, Member, Participant, UserId};
use crate::test::mock_data;
use crate::util::jid_ext::BareJidExt;

impl RoomInternals {
    pub fn direct_message(jid: UserId) -> Self {
        let jid = jid.into();

        Self::for_direct_message(&jid, &jid.formatted_username())
    }

    pub fn mock_pending_room(jid: impl Into<RoomId>, next_hash: &str) -> Self {
        Self::pending(
            &jid.into(),
            &format!(
                "{}-{}",
                mock_data::account_jid().node_str().unwrap(),
                next_hash
            ),
        )
    }

    pub fn group(jid: impl Into<RoomId>) -> Self {
        Self::new(RoomInfo {
            room_id: jid.into(),
            description: None,
            user_nickname: mock_data::account_jid().node_str().unwrap().to_string(),
            r#type: RoomType::Group,
        })
    }

    pub fn public_channel(jid: impl Into<RoomId>) -> Self {
        Self::new(RoomInfo {
            room_id: jid.into(),
            description: None,
            user_nickname: mock_data::account_jid().node_str().unwrap().to_string(),
            r#type: RoomType::PublicChannel,
        })
    }

    pub fn private_channel(jid: impl Into<RoomId>) -> Self {
        Self::new(RoomInfo {
            room_id: jid.into(),
            description: None,
            user_nickname: mock_data::account_jid().node_str().unwrap().to_string(),
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

    pub fn with_occupants(self, occupant: impl IntoIterator<Item = (Jid, Participant)>) -> Self {
        self.set_participants(occupant.into_iter().collect());
        self
    }
}

impl Participant {
    pub fn owner() -> Self {
        Participant {
            id: None,
            name: None,
            affiliation: RoomAffiliation::Owner,
            compose_state: Default::default(),
            compose_state_updated: Default::default(),
            availability: Availability::Unavailable,
        }
    }

    pub fn member() -> Self {
        Participant {
            id: None,
            name: None,
            affiliation: RoomAffiliation::Owner,
            compose_state: Default::default(),
            compose_state_updated: Default::default(),
            availability: Availability::Unavailable,
        }
    }

    pub fn set_real_id(mut self, jid: &UserId) -> Self {
        self.id = Some(jid.clone());
        self
    }

    pub fn set_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn set_compose_state(mut self, compose_state: ComposeState) -> Self {
        self.compose_state = compose_state;
        self
    }

    pub fn set_compose_state_updated(mut self, timestamp: DateTime<Utc>) -> Self {
        self.compose_state_updated = timestamp;
        self
    }
}
