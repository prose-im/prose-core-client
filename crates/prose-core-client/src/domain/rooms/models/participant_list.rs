// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::domain::shared::models::{
    AnonOccupantId, Availability, ParticipantId, UserBasicInfo, UserId,
};

use super::{ComposeState, RoomAffiliation, RoomSessionParticipant};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ParticipantList {
    anon_occupant_id_to_participant_id_map: HashMap<AnonOccupantId, ParticipantId>,
    participants_map: HashMap<ParticipantId, Participant>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Participant {
    /// The real JID of the occupant. Only available in non-anonymous rooms.
    pub real_id: Option<UserId>,
    pub anon_occupant_id: Option<AnonOccupantId>,
    pub name: Option<String>,
    pub is_self: bool,
    pub affiliation: RoomAffiliation,
    pub availability: Availability,
    pub compose_state: ComposeState,
    pub compose_state_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegisteredMember {
    pub user_id: UserId,
    pub affiliation: RoomAffiliation,
    pub name: Option<String>,
    pub is_self: bool,
}

impl ParticipantList {
    pub fn for_direct_message(
        contact_id: &UserId,
        contact_name: &str,
        availability: Availability,
    ) -> Self {
        Self {
            anon_occupant_id_to_participant_id_map: Default::default(),
            participants_map: HashMap::from([(
                ParticipantId::User(contact_id.clone()),
                Participant {
                    real_id: Some(contact_id.clone()),
                    anon_occupant_id: None,
                    name: Some(contact_name.to_string()),
                    is_self: false,
                    affiliation: RoomAffiliation::Owner,
                    availability,
                    compose_state: Default::default(),
                    compose_state_updated: Default::default(),
                },
            )]),
        }
    }

    pub fn new(
        members: impl IntoIterator<Item = RegisteredMember>,
        participants: impl IntoIterator<Item = RoomSessionParticipant>,
    ) -> Self {
        let mut anon_occupant_id_to_participant_id_map = HashMap::new();
        let mut participants_map = HashMap::new();
        let mut real_to_occupant_id_map = HashMap::new();

        for p in participants {
            if let Some(real_id) = &p.real_id {
                real_to_occupant_id_map.insert(real_id.clone(), p.id.clone());
            }

            let id = ParticipantId::Occupant(p.id);

            if let Some(anon_id) = &p.anon_id {
                anon_occupant_id_to_participant_id_map.insert(anon_id.clone(), id.clone());
            }

            let participant = Participant {
                real_id: p.real_id,
                anon_occupant_id: p.anon_id,
                name: None,
                is_self: p.is_self,
                affiliation: p.affiliation,
                availability: p.availability,
                compose_state: ComposeState::Idle,
                compose_state_updated: Default::default(),
            };

            participants_map.insert(id, participant);
        }

        for member in members {
            let participant_id = real_to_occupant_id_map
                .get(&member.user_id)
                .cloned()
                .map(ParticipantId::Occupant)
                .unwrap_or_else(|| ParticipantId::User(member.user_id.clone()));

            participants_map
                .entry(participant_id)
                .and_modify(|p| p.name = member.name.clone())
                .or_insert_with(|| Participant {
                    real_id: Some(member.user_id),
                    anon_occupant_id: None,
                    name: member.name,
                    is_self: member.is_self,
                    affiliation: member.affiliation,
                    availability: Availability::Unavailable,
                    compose_state: ComposeState::Idle,
                    compose_state_updated: Default::default(),
                });
        }

        Self {
            anon_occupant_id_to_participant_id_map,
            participants_map,
        }
    }

    /// Modifies the participant's availability or inserts a new participant with the availability
    /// if it didn't exist.
    pub fn set_availability(
        &mut self,
        id: &ParticipantId,
        is_self: bool,
        availability: &Availability,
    ) {
        self.participants_map
            .entry(id.clone())
            .and_modify(|participant| {
                participant.availability = availability.clone();
                participant.is_self = is_self;
                if availability == &Availability::Unavailable {
                    participant.compose_state = ComposeState::Idle;
                }
            })
            .or_insert_with(|| Participant {
                real_id: None,
                anon_occupant_id: None,
                name: None,
                is_self,
                affiliation: RoomAffiliation::None,
                availability: availability.clone(),
                compose_state: ComposeState::Idle,
                compose_state_updated: DateTime::default(),
            });
    }

    /// Sets the participant's affiliation. Does nothing if the participant doesn't exist.
    pub fn set_affiliation(
        &mut self,
        id: &ParticipantId,
        is_self: bool,
        affiliation: &RoomAffiliation,
    ) {
        self.participants_map
            .entry(id.clone())
            .and_modify(|participant| {
                participant.affiliation = affiliation.clone();
                participant.is_self = is_self;
            })
            .or_insert_with(|| Participant {
                real_id: None,
                anon_occupant_id: None,
                name: None,
                is_self,
                affiliation: affiliation.clone(),
                availability: Availability::Unavailable,
                compose_state: ComposeState::Idle,
                compose_state_updated: DateTime::default(),
            });
    }

    /// Sets the participant's compose state. Does nothing if the participant doesn't exist.
    pub fn set_compose_state(
        &mut self,
        id: &ParticipantId,
        timestamp: &DateTime<Utc>,
        compose_state: ComposeState,
    ) {
        self.participants_map
            .entry(id.clone())
            .and_modify(|participant| {
                participant.compose_state = compose_state;
                participant.compose_state_updated = timestamp.clone()
            });
    }

    pub fn add_user(
        &mut self,
        real_id: &UserId,
        is_self: bool,
        affiliation: &RoomAffiliation,
        name: Option<&str>,
    ) {
        if self
            .participants_map
            .values()
            .find(|p| p.real_id.as_ref() == Some(real_id))
            .is_some()
        {
            return;
        }

        self.participants_map
            .entry(ParticipantId::User(real_id.clone()))
            .and_modify(|participant| {
                participant.affiliation = affiliation.clone();
                participant.name = name.map(ToString::to_string);
                participant.is_self = is_self;
            })
            .or_insert_with(|| Participant {
                real_id: Some(real_id.clone()),
                anon_occupant_id: None,
                name: name.map(ToString::to_string),
                is_self,
                affiliation: affiliation.clone(),
                availability: Availability::Unavailable,
                compose_state: ComposeState::Idle,
                compose_state_updated: DateTime::default(),
            });
    }

    /// Sets the participant's real id, anonymous occupant id and name. Does nothing if the
    /// participant doesn't exist.
    pub fn set_ids_and_name(
        &mut self,
        id: &ParticipantId,
        real_id: Option<&UserId>,
        anon_occupant_id: Option<&AnonOccupantId>,
        name: Option<&str>,
    ) {
        let Some(participant) = self.participants_map.get_mut(id) else {
            return;
        };

        participant.real_id = real_id.cloned();
        participant.anon_occupant_id = anon_occupant_id.cloned();
        participant.name = name.map(ToString::to_string);

        // Remove registered user matching the real id…
        if let Some(real_id) = real_id {
            self.participants_map
                .remove(&ParticipantId::User(real_id.clone()));
        }

        self.anon_occupant_id_to_participant_id_map
            .retain(|_, participant_id| participant_id != id);

        if let Some(anon_occupant_id) = anon_occupant_id {
            self.anon_occupant_id_to_participant_id_map
                .insert(anon_occupant_id.clone(), id.clone());
        }
    }

    pub fn get_user_id(&self, anon_occupant_id: &AnonOccupantId) -> Option<UserId> {
        let Some(participant_id) = self
            .anon_occupant_id_to_participant_id_map
            .get(anon_occupant_id)
        else {
            return None;
        };

        self.participants_map
            .get(participant_id)
            .and_then(|participant| participant.real_id.clone())
    }

    /// Removes the participant. Does nothing if the participant doesn't exist.
    pub fn remove(&mut self, id: &ParticipantId) {
        self.participants_map.remove(id);
    }

    /// Returns the participant identified by `id` if it exists.
    pub fn get(&self, id: &ParticipantId) -> Option<&Participant> {
        self.participants_map.get(id)
    }

    /// Returns an iterator over the contained key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&ParticipantId, &Participant)> {
        self.participants_map.iter()
    }

    /// Returns an iterator over the contained participants.
    pub fn values(&self) -> impl Iterator<Item = &Participant> {
        self.participants_map.values()
    }

    /// Returns the number of participants.
    pub fn len(&self) -> usize {
        self.participants_map.len()
    }
}

impl ParticipantList {
    /// Returns the real JIDs of all composing users that started composing after `started_after`.
    /// If we don't have a real JID for a composing user they are excluded from the list.
    pub fn composing_users(&self, started_after: DateTime<Utc>) -> Vec<UserBasicInfo> {
        let mut composing_occupants = self
            .participants_map
            .values()
            .filter_map(|occupant| {
                if occupant.compose_state != ComposeState::Composing
                    || occupant.compose_state_updated <= started_after
                    || occupant.real_id.is_none()
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
                let Some(real_id) = &occupant.real_id else {
                    return None;
                };

                Some(UserBasicInfo {
                    name: occupant
                        .name
                        .clone()
                        .unwrap_or_else(|| real_id.formatted_username()),
                    id: real_id.clone(),
                })
            })
            .collect()
    }
}

#[cfg(feature = "test")]
impl ParticipantList {
    pub fn extend_participants(&mut self, participants: HashMap<ParticipantId, Participant>) {
        self.participants_map.extend(participants);
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use crate::domain::shared::models::OccupantId;
    use crate::{occupant_id, user_id};

    use super::*;

    #[test]
    fn test_insert_occupant() {
        let mut state = ParticipantList::default();
        assert!(state.participants_map.is_empty());

        state.set_availability(
            &occupant_id!("room@prose.org/a").into(),
            false,
            &Availability::Unavailable,
        );
        state.set_affiliation(
            &occupant_id!("room@prose.org/a").into(),
            false,
            &RoomAffiliation::Owner,
        );
        state.set_ids_and_name(
            &occupant_id!("room@prose.org/a").into(),
            Some(&user_id!("a@prose.org")),
            None,
            None,
        );

        state.set_availability(
            &user_id!("b@prose.org").into(),
            false,
            &Availability::Unavailable,
        );
        state.set_affiliation(
            &user_id!("b@prose.org").into(),
            false,
            &RoomAffiliation::Member,
        );

        assert_eq!(state.participants_map.len(), 2);
        assert_eq!(
            state
                .participants_map
                .get(&occupant_id!("room@prose.org/a").into())
                .unwrap(),
            &Participant {
                real_id: Some(user_id!("a@prose.org")),
                affiliation: RoomAffiliation::Owner,
                ..Default::default()
            }
        );
        assert_eq!(
            state
                .participants_map
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
        let mut state = ParticipantList::default();

        state.set_availability(
            &occupant_id!("room@prose.org/a").into(),
            false,
            &Availability::Unavailable,
        );

        state.set_compose_state(
            &occupant_id!("room@prose.org/a").into(),
            &Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 0).unwrap(),
            ComposeState::Composing,
        );

        assert_eq!(
            state
                .participants_map
                .get(&occupant_id!("room@prose.org/a").into())
                .unwrap()
                .compose_state,
            ComposeState::Composing
        );
        assert_eq!(
            state
                .participants_map
                .get(&occupant_id!("room@prose.org/a").into())
                .unwrap()
                .compose_state_updated,
            Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_composing_users() {
        let mut state = ParticipantList::default();

        state.participants_map.insert(
            occupant_id!("room@prose.org/a").into(),
            Participant {
                real_id: Some(user_id!("a@prose.org")),
                compose_state: ComposeState::Composing,
                compose_state_updated: Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 30).unwrap(),
                ..Default::default()
            },
        );
        state.participants_map.insert(
            occupant_id!("room@prose.org/b").into(),
            Participant {
                real_id: Some(user_id!("b@prose.org")),
                compose_state: ComposeState::Idle,
                compose_state_updated: Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 30).unwrap(),
                ..Default::default()
            },
        );
        state.participants_map.insert(
            occupant_id!("room@prose.org/c").into(),
            Participant {
                real_id: Some(user_id!("c@prose.org")),
                name: Some("Jonathan Doe".to_string()),
                compose_state: ComposeState::Composing,
                compose_state_updated: Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 20).unwrap(),
                ..Default::default()
            },
        );
        state.participants_map.insert(
            occupant_id!("room@prose.org/d").into(),
            Participant {
                real_id: Some(user_id!("d@prose.org")),
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

    #[test]
    fn test_registered_members_in_muc_room() {
        // Start with a fresh state…
        let mut list = ParticipantList::new(
            vec![
                RegisteredMember {
                    user_id: user_id!("a@prose.org"),
                    affiliation: RoomAffiliation::Member,
                    name: Some("User A".to_string()),
                    is_self: false,
                },
                RegisteredMember {
                    user_id: user_id!("b@prose.org"),
                    affiliation: RoomAffiliation::Member,
                    name: Some("User B".to_string()),
                    is_self: false,
                },
            ],
            // Assume that a registered member is online and we've received their presence when
            // connecting to the room.
            vec![RoomSessionParticipant {
                id: occupant_id!("room@conference.prose.org/a"),
                is_self: false,
                anon_id: None,
                real_id: Some(user_id!("a@prose.org")),
                affiliation: RoomAffiliation::Member,
                availability: Availability::Available,
            }],
        );

        assert_eq!(
            list.participants_map,
            HashMap::from([
                (
                    ParticipantId::Occupant(occupant_id!("room@conference.prose.org/a")),
                    Participant {
                        real_id: Some(user_id!("a@prose.org")),
                        anon_occupant_id: None,
                        name: Some("User A".to_string()),
                        is_self: false,
                        affiliation: RoomAffiliation::Member,
                        availability: Availability::Available,
                        compose_state: Default::default(),
                        compose_state_updated: Default::default(),
                    }
                ),
                (
                    ParticipantId::User(user_id!("b@prose.org")),
                    Participant {
                        real_id: Some(user_id!("b@prose.org")),
                        anon_occupant_id: None,
                        name: Some("User B".to_string()),
                        is_self: false,
                        affiliation: RoomAffiliation::Member,
                        availability: Availability::Unavailable,
                        compose_state: Default::default(),
                        compose_state_updated: Default::default(),
                    }
                )
            ])
        );

        // Now the second member comes online…
        list.set_availability(
            &ParticipantId::Occupant(occupant_id!("room@conference.prose.org/b")),
            false,
            &Availability::Available,
        );
        list.set_affiliation(
            &ParticipantId::Occupant(occupant_id!("room@conference.prose.org/b")),
            false,
            &RoomAffiliation::Member,
        );
        list.set_ids_and_name(
            &ParticipantId::Occupant(occupant_id!("room@conference.prose.org/b")),
            Some(&user_id!("b@prose.org")),
            None,
            Some("User B New Name"),
        );

        assert_eq!(
            list.participants_map,
            HashMap::from([
                (
                    ParticipantId::Occupant(occupant_id!("room@conference.prose.org/a")),
                    Participant {
                        real_id: Some(user_id!("a@prose.org")),
                        anon_occupant_id: None,
                        name: Some("User A".to_string()),
                        is_self: false,
                        affiliation: RoomAffiliation::Member,
                        availability: Availability::Available,
                        compose_state: Default::default(),
                        compose_state_updated: Default::default(),
                    }
                ),
                (
                    ParticipantId::Occupant(occupant_id!("room@conference.prose.org/b")),
                    Participant {
                        real_id: Some(user_id!("b@prose.org")),
                        anon_occupant_id: None,
                        name: Some("User B New Name".to_string()),
                        is_self: false,
                        affiliation: RoomAffiliation::Member,
                        availability: Availability::Available,
                        compose_state: Default::default(),
                        compose_state_updated: Default::default(),
                    }
                )
            ])
        );
    }

    #[test]
    fn test_registered_members_in_direct_message_room() {
        // Start with a fresh state…
        let mut list = ParticipantList::for_direct_message(
            &user_id!("a@prose.org"),
            "User A",
            Availability::Unavailable,
        );

        assert_eq!(
            list.participants_map,
            HashMap::from([(
                ParticipantId::User(user_id!("a@prose.org")),
                Participant {
                    real_id: Some(user_id!("a@prose.org")),
                    anon_occupant_id: None,
                    name: Some("User A".to_string()),
                    is_self: false,
                    affiliation: RoomAffiliation::Owner,
                    availability: Availability::Unavailable,
                    compose_state: Default::default(),
                    compose_state_updated: Default::default(),
                }
            ),])
        );

        // Now the user comes online…
        list.set_availability(
            &ParticipantId::User(user_id!("a@prose.org")),
            false,
            &Availability::Available,
        );

        assert_eq!(
            list.participants_map,
            HashMap::from([(
                ParticipantId::User(user_id!("a@prose.org")),
                Participant {
                    real_id: Some(user_id!("a@prose.org")),
                    anon_occupant_id: None,
                    name: Some("User A".to_string()),
                    is_self: false,
                    affiliation: RoomAffiliation::Owner,
                    availability: Availability::Available,
                    compose_state: Default::default(),
                    compose_state_updated: Default::default(),
                }
            ),])
        );
    }
}
