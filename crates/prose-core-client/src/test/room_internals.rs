// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};

use crate::domain::rooms::models::{
    ComposeState, ParticipantList, RegisteredMember, Room, RoomAffiliation, RoomInfo,
    RoomSidebarState,
};
use crate::domain::settings::models::SyncedRoomSettings;
use crate::domain::shared::models::{ParticipantId, RoomId, RoomType};
use crate::domain::user_info::models::Presence;
use crate::dtos::{Availability, Participant, RoomState, UserId};
use crate::test::mock_data;

impl Room {
    pub fn direct_message(jid: UserId, availability: Availability) -> Self {
        let jid = jid.into();

        Self::for_direct_message(
            &jid,
            &jid.formatted_username(),
            Presence {
                availability,
                ..Default::default()
            },
            RoomSidebarState::InSidebar,
            Default::default(),
            SyncedRoomSettings::new(RoomId::User(jid.clone())),
        )
    }

    pub fn mock_connecting_room(jid: impl Into<RoomId>, next_hash: &str) -> Self {
        Self::connecting(
            &jid.into(),
            &format!("{}-{}", mock_data::account_jid().username(), next_hash),
            RoomSidebarState::InSidebar,
        )
    }

    pub fn group(jid: impl Into<RoomId>) -> Self {
        Self::mock(RoomInfo {
            room_id: jid.into(),
            user_nickname: mock_data::account_jid().username().to_string(),
            r#type: RoomType::Group,
            features: Default::default(),
        })
    }

    pub fn public_channel(jid: impl Into<RoomId>) -> Self {
        Self::mock(RoomInfo {
            room_id: jid.into(),
            user_nickname: mock_data::account_jid().username().to_string(),
            r#type: RoomType::PublicChannel,
            features: Default::default(),
        })
    }

    pub fn private_channel(jid: impl Into<RoomId>) -> Self {
        Self::mock(RoomInfo {
            room_id: jid.into(),
            user_nickname: mock_data::account_jid().username().to_string(),
            r#type: RoomType::PrivateChannel,
            features: Default::default(),
        })
    }

    pub fn with_name(self, name: impl AsRef<str>) -> Self {
        self.set_name(Some(name.as_ref().to_string()));
        self
    }

    pub fn with_topic(self, topic: Option<&str>) -> Self {
        self.set_topic(topic.map(ToString::to_string));
        self
    }

    pub fn with_members(self, members: impl IntoIterator<Item = RegisteredMember>) -> Self {
        self.with_participants_mut(|p| {
            *p = ParticipantList::new(members, []);
        });
        self
    }

    pub fn by_adding_participants<Id: Into<ParticipantId>>(
        self,
        occupant: impl IntoIterator<Item = (Id, Participant)>,
    ) -> Self {
        self.with_participants_mut(|p| {
            p.extend_participants(occupant.into_iter().map(|(id, p)| (id.into(), p)).collect())
        });
        self
    }

    pub fn with_sidebar_state(self, state: RoomSidebarState) -> Self {
        self.set_sidebar_state(state);
        self
    }

    pub fn with_state(self, state: RoomState) -> Self {
        self.set_state(state);
        self
    }

    pub fn is_disconnected(&self) -> DisconnectedState {
        match self.state() {
            RoomState::Pending | RoomState::Connecting | RoomState::Connected => {
                DisconnectedState {
                    is_disconnected: false,
                    can_retry: false,
                }
            }
            RoomState::Disconnected { can_retry, .. } => DisconnectedState {
                is_disconnected: true,
                can_retry,
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DisconnectedState {
    pub is_disconnected: bool,
    pub can_retry: bool,
}

impl Participant {
    pub fn owner() -> Self {
        Participant {
            real_id: None,
            name: None,
            is_self: false,
            affiliation: RoomAffiliation::Owner,
            compose_state: Default::default(),
            compose_state_updated: Default::default(),
            availability: Availability::Unavailable,
            anon_occupant_id: None,
            avatar: None,
            client: None,
            caps: None,
        }
    }

    pub fn member() -> Self {
        Participant {
            real_id: None,
            anon_occupant_id: None,
            name: None,
            is_self: false,
            affiliation: RoomAffiliation::Owner,
            compose_state: Default::default(),
            compose_state_updated: Default::default(),
            availability: Availability::Unavailable,
            avatar: None,
            client: None,
            caps: None,
        }
    }

    pub fn set_real_id(mut self, id: &UserId) -> Self {
        self.real_id = Some(id.clone());
        self
    }

    pub fn set_availability(mut self, availability: Availability) -> Self {
        self.availability = availability;
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
