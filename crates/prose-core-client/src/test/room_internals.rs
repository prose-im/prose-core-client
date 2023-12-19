// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};

use crate::domain::rooms::models::{
    ComposeState, RegisteredMember, RoomAffiliation, RoomInfo, RoomInternals,
};
use crate::domain::shared::models::{ParticipantId, RoomId, RoomType};
use crate::dtos::{Availability, Participant, UserId};
use crate::test::mock_data;

impl RoomInternals {
    pub fn direct_message(jid: UserId, availability: &Availability) -> Self {
        let jid = jid.into();

        Self::for_direct_message(&jid, &jid.formatted_username(), availability)
    }

    pub fn mock_pending_room(jid: impl Into<RoomId>, next_hash: &str) -> Self {
        Self::pending(
            &jid.into(),
            &format!("{}-{}", mock_data::account_jid().username(), next_hash),
        )
    }

    pub fn group(jid: impl Into<RoomId>) -> Self {
        Self::new(RoomInfo {
            room_id: jid.into(),
            user_nickname: mock_data::account_jid().username().to_string(),
            r#type: RoomType::Group,
        })
    }

    pub fn public_channel(jid: impl Into<RoomId>) -> Self {
        Self::new(RoomInfo {
            room_id: jid.into(),
            user_nickname: mock_data::account_jid().username().to_string(),
            r#type: RoomType::PublicChannel,
        })
    }

    pub fn private_channel(jid: impl Into<RoomId>) -> Self {
        Self::new(RoomInfo {
            room_id: jid.into(),
            user_nickname: mock_data::account_jid().username().to_string(),
            r#type: RoomType::PrivateChannel,
        })
    }

    pub fn with_user_nickname(mut self, nickname: impl Into<String>) -> Self {
        self.user_nickname = nickname.into();
        self
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
        self.participants_mut().set_registered_members(members);
        self
    }

    pub fn with_participants<Id: Into<ParticipantId>>(
        self,
        occupant: impl IntoIterator<Item = (Id, Participant)>,
    ) -> Self {
        self.participants_mut()
            .extend_participants(occupant.into_iter().map(|(id, p)| (id.into(), p)).collect());
        self
    }
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
        }
    }

    pub fn set_real_id(mut self, id: &UserId) -> Self {
        self.real_id = Some(id.clone());
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
