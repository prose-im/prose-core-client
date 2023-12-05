// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::domain::rooms::models::{ComposeState, RoomAffiliation};
use crate::domain::shared::models::{
    Availability, OccupantId, ParticipantId, UserBasicInfo, UserId,
};

#[derive(Default, Clone, Debug, PartialEq)]
pub struct RoomState {
    /// The name of the room.
    pub name: Option<String>,
    /// The room's topic.
    pub topic: Option<String>,
    /// The list of members. Only available for DirectMessage and Group (member-only rooms).
    pub members: HashMap<UserId, RoomMember>,
    /// The participants in the room.
    pub participants: HashMap<ParticipantId, Participant>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RoomMember {
    pub name: String,
    pub affiliation: RoomAffiliation,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Participant {
    /// The real JID of the occupant. Only available in non-anonymous rooms.
    pub id: Option<UserId>,
    pub name: Option<String>,
    pub affiliation: RoomAffiliation,
    pub availability: Availability,
    pub compose_state: ComposeState,
    pub compose_state_updated: DateTime<Utc>,
}

impl RoomState {
    pub fn insert_participant(
        &mut self,
        id: &ParticipantId,
        real_id: Option<&UserId>,
        name: Option<&str>,
        affiliation: &RoomAffiliation,
        availability: &Availability,
    ) {
        let participant = self.participants.entry(id.clone()).or_default();
        participant.id = real_id.cloned();
        participant.name = name.map(ToString::to_string);
        participant.affiliation = affiliation.clone();
        participant.availability = availability.clone();
    }

    pub fn set_participant_compose_state(
        &mut self,
        id: &ParticipantId,
        timestamp: &DateTime<Utc>,
        compose_state: ComposeState,
    ) {
        self.participants
            .entry(id.clone())
            .and_modify(|participant| {
                participant.compose_state = compose_state;
                participant.compose_state_updated = timestamp.clone()
            });
    }

    pub fn set_participant_availability(
        &mut self,
        id: &ParticipantId,
        availability: &Availability,
    ) {
        self.participants
            .entry(id.clone())
            .and_modify(|participant| participant.availability = availability.clone());
    }

    pub fn set_participant_affiliation(
        &mut self,
        id: &ParticipantId,
        affiliation: &RoomAffiliation,
    ) {
        self.participants
            .entry(id.clone())
            .and_modify(|participant| participant.affiliation = affiliation.clone());
    }

    /// Returns the real JIDs of all composing users that started composing after `started_after`.
    /// If we don't have a real JID for a composing user they are excluded from the list.
    pub fn composing_users(&self, started_after: DateTime<Utc>) -> Vec<UserBasicInfo> {
        let mut composing_occupants = self
            .participants
            .values()
            .filter_map(|occupant| {
                if occupant.compose_state != ComposeState::Composing
                    || occupant.compose_state_updated <= started_after
                    || occupant.id.is_none()
                {
                    return None;
                }
                Some(occupant.clone())
            })
            .collect::<Vec<_>>();

        composing_occupants.sort_by_key(|o| o.compose_state_updated);

        composing_occupants
            .into_iter()
            .filter_map(|occupant| {
                let Some(jid) = &occupant.id else {
                    return None;
                };

                Some(UserBasicInfo {
                    name: occupant
                        .name
                        .clone()
                        .unwrap_or_else(|| jid.formatted_username()),
                    id: jid.clone(),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use crate::{occupant_id, user_id};

    use super::*;

    #[test]
    fn test_insert_occupant() {
        let mut state = RoomState::default();
        assert!(state.participants.is_empty());

        state.insert_participant(
            &occupant_id!("room@prose.org/a").into(),
            Some(&user_id!("a@prose.org")),
            None,
            &RoomAffiliation::Owner,
            &Availability::Unavailable,
        );
        state.insert_participant(
            &user_id!("b@prose.org").into(),
            None,
            None,
            &RoomAffiliation::Member,
            &Availability::Unavailable,
        );

        assert_eq!(state.participants.len(), 2);
        assert_eq!(
            state
                .participants
                .get(&occupant_id!("room@prose.org/a").into())
                .unwrap(),
            &Participant {
                id: Some(user_id!("a@prose.org")),
                affiliation: RoomAffiliation::Owner,
                ..Default::default()
            }
        );
        assert_eq!(
            state
                .participants
                .get(&user_id!("b@prose.org").into())
                .unwrap(),
            &Participant {
                affiliation: RoomAffiliation::Member,
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_set_occupant_chat_state() {
        let mut state = RoomState::default();

        state.insert_participant(
            &occupant_id!("room@prose.org/a").into(),
            Some(&user_id!("a@prose.org")),
            None,
            &RoomAffiliation::Owner,
            &Availability::Unavailable,
        );

        state.set_participant_compose_state(
            &occupant_id!("room@prose.org/a").into(),
            &Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 0).unwrap(),
            ComposeState::Composing,
        );

        assert_eq!(
            state
                .participants
                .get(&occupant_id!("room@prose.org/a").into())
                .unwrap()
                .compose_state,
            ComposeState::Composing
        );
        assert_eq!(
            state
                .participants
                .get(&occupant_id!("room@prose.org/a").into())
                .unwrap()
                .compose_state_updated,
            Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_composing_users() {
        let mut state = RoomState::default();

        state.participants.insert(
            occupant_id!("room@prose.org/a").into(),
            Participant {
                id: Some(user_id!("a@prose.org")),
                compose_state: ComposeState::Composing,
                compose_state_updated: Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 30).unwrap(),
                ..Default::default()
            },
        );
        state.participants.insert(
            occupant_id!("room@prose.org/b").into(),
            Participant {
                id: Some(user_id!("b@prose.org")),
                compose_state: ComposeState::Idle,
                compose_state_updated: Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 30).unwrap(),
                ..Default::default()
            },
        );
        state.participants.insert(
            occupant_id!("room@prose.org/c").into(),
            Participant {
                id: Some(user_id!("c@prose.org")),
                name: Some("Jonathan Doe".to_string()),
                compose_state: ComposeState::Composing,
                compose_state_updated: Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 20).unwrap(),
                ..Default::default()
            },
        );
        state.participants.insert(
            occupant_id!("room@prose.org/d").into(),
            Participant {
                id: Some(user_id!("d@prose.org")),
                compose_state: ComposeState::Composing,
                compose_state_updated: Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 10).unwrap(),
                ..Default::default()
            },
        );

        assert_eq!(
            state.composing_users(Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 10).unwrap()),
            vec![
                UserBasicInfo {
                    name: "Jonathan Doe".to_string(),
                    id: user_id!("c@prose.org")
                },
                UserBasicInfo {
                    name: "A".to_string(),
                    id: user_id!("a@prose.org")
                },
            ]
        );
    }
}
