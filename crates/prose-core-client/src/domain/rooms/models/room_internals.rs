// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use chrono::{DateTime, Utc};
use jid::FullJid;
use parking_lot::RwLock;

use crate::domain::rooms::models::{ComposeState, RoomAffiliation, RoomMember, RoomState};
use crate::domain::shared::models::{
    Availability, ParticipantId, RoomId, RoomType, UserBasicInfo, UserId,
};

use super::Participant;

/// Contains information about a connected room and its state.
#[derive(Debug)]
pub struct RoomInternals {
    info: RoomInfo,
    state: RwLock<RoomState>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoomInfo {
    /// The JID of the room.
    pub room_id: RoomId,
    /// The description of the room.
    pub description: Option<String>,
    /// The nickname with which our user is connected to the room.
    pub user_nickname: String,
    /// The type of the room.
    pub r#type: RoomType,
}

impl Deref for RoomInternals {
    type Target = RoomInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl DerefMut for RoomInternals {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.info
    }
}

impl RoomInternals {
    pub fn name(&self) -> Option<String> {
        self.state.read().name.clone()
    }

    pub fn set_name(&self, name: &str) {
        self.state.write().name.replace(name.to_string());
    }

    pub fn topic(&self) -> Option<String> {
        self.state.read().topic.clone()
    }

    pub fn set_topic(&self, topic: Option<String>) {
        self.state.write().topic = topic
    }

    pub fn participants(&self) -> Vec<Participant> {
        self.state.read().participants.values().cloned().collect()
    }

    pub fn members(&self) -> Vec<(UserId, RoomMember)> {
        self.state
            .read()
            .members
            .iter()
            .map(|(id, member)| (id.clone(), member.clone()))
            .collect()
    }

    pub fn get_participant(&self, id: &ParticipantId) -> Option<Participant> {
        self.state.read().participants.get(&id).cloned()
    }

    pub fn insert_participant(
        &self,
        id: &ParticipantId,
        real_id: Option<&UserId>,
        name: Option<&str>,
        affiliation: &RoomAffiliation,
        availability: &Availability,
    ) {
        self.state
            .write()
            .insert_participant(id, real_id, name, affiliation, availability)
    }

    pub fn remove_participant(&self, id: &ParticipantId) {
        self.state.write().participants.remove(id);
    }

    pub fn set_participant_compose_state(
        &self,
        id: &ParticipantId,
        timestamp: &DateTime<Utc>,
        compose_state: ComposeState,
    ) {
        self.state
            .write()
            .set_participant_compose_state(id, timestamp, compose_state)
    }

    pub fn set_participant_availability(&self, id: &ParticipantId, availability: &Availability) {
        self.state
            .write()
            .set_participant_availability(id, availability)
    }

    pub fn set_participant_affiliation(&self, id: &ParticipantId, affiliation: &RoomAffiliation) {
        self.state
            .write()
            .set_participant_affiliation(id, affiliation)
    }

    pub fn set_participant_real_id_and_name(
        &self,
        id: &ParticipantId,
        real_id: Option<&UserId>,
        name: Option<&str>,
    ) {
        self.state
            .write()
            .set_participant_real_id_and_name(id, real_id, name)
    }

    /// Returns the real JIDs of all composing users that started composing after `started_after`.
    /// If we don't have a real JID for a composing user they are excluded from the list.
    pub fn composing_users(&self, started_after: DateTime<Utc>) -> Vec<UserBasicInfo> {
        self.state.read().composing_users(started_after)
    }
}

impl RoomInternals {
    pub fn pending(room_jid: &RoomId, nickname: &str) -> Self {
        Self {
            info: RoomInfo {
                room_id: room_jid.clone(),
                description: None,
                user_nickname: nickname.to_string(),
                r#type: RoomType::Pending,
            },
            state: Default::default(),
        }
    }

    pub fn is_pending(&self) -> bool {
        self.info.r#type == RoomType::Pending
    }

    // Resolves a pending room.
    pub fn by_resolving_with_info(
        &self,
        name: Option<String>,
        info: RoomInfo,
        members: HashMap<UserId, RoomMember>,
    ) -> Self {
        assert!(self.is_pending(), "Cannot promote a non-pending room");

        let mut state = self.state.read().clone();
        state.name = name;

        // TODO: Merge members
        todo!("Merge members");

        Self {
            info,
            state: RwLock::new(state),
        }
    }
}

impl RoomInternals {
    pub fn for_direct_message(contact_id: &UserId, contact_name: &str) -> Self {
        Self {
            info: RoomInfo {
                room_id: RoomId::from(contact_id.clone().into_inner()),
                description: None,
                user_nickname: "no_nickname".to_string(),
                r#type: RoomType::DirectMessage,
            },
            state: RwLock::new(RoomState {
                name: Some(contact_name.to_string()),
                topic: None,
                members: HashMap::from([(
                    contact_id.clone(),
                    RoomMember {
                        name: contact_name.to_string(),
                        affiliation: RoomAffiliation::Owner,
                    },
                )]),
                participants: HashMap::from([(
                    contact_id.clone().into(),
                    Participant {
                        id: Some(contact_id.clone()),
                        name: Some(contact_name.to_string()),
                        affiliation: RoomAffiliation::Owner,
                        availability: Availability::Unavailable,
                        compose_state: ComposeState::Idle,
                        compose_state_updated: Default::default(),
                    },
                )]),
            }),
        }
    }
}

#[cfg(feature = "test")]
impl RoomInternals {
    pub fn set_participants(&self, participants: HashMap<ParticipantId, Participant>) {
        self.state.write().participants = participants;
    }

    pub fn set_members(&self, members: HashMap<UserId, RoomMember>) {
        self.state.write().members = members;
    }

    pub fn new(info: RoomInfo) -> Self {
        Self {
            info,
            state: Default::default(),
        }
    }
}

impl RoomInfo {
    /// Returns the full jid of the connected user by appending their nickname to the room's
    /// bare jid.
    pub fn user_full_jid(&self) -> FullJid {
        self.room_id.with_resource_str(&self.user_nickname)
            .expect("The provided JID and user_nickname were invalid and could not be used to form a FullJid.")
    }
}

#[cfg(feature = "test")]
impl PartialEq for RoomInternals {
    fn eq(&self, other: &Self) -> bool {
        self.info == other.info && *self.state.read() == *other.state.read()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use prose_xmpp::bare;

    use crate::dtos::Participant;
    use crate::user_id;

    use super::*;

    #[test]
    fn test_room_internals_for_direct_message() {
        let internals =
            RoomInternals::for_direct_message(&user_id!("contact@prose.org"), "Jane Doe");

        assert_eq!(
            internals,
            RoomInternals {
                info: RoomInfo {
                    room_id: bare!("contact@prose.org").into(),
                    description: None,
                    user_nickname: "no_nickname".to_string(),
                    r#type: RoomType::DirectMessage,
                },
                state: RwLock::new(RoomState {
                    name: Some("Jane Doe".to_string()),
                    topic: None,
                    members: HashMap::from([(
                        user_id!("contact@prose.org"),
                        RoomMember {
                            name: "Jane Doe".to_string(),
                            affiliation: RoomAffiliation::Owner,
                        }
                    )]),
                    participants: HashMap::from([(
                        user_id!("contact@prose.org").into(),
                        Participant {
                            id: Some(user_id!("contact@prose.org")),
                            name: Some("Jane Doe".to_string()),
                            affiliation: RoomAffiliation::Owner,
                            availability: Default::default(),
                            compose_state: ComposeState::Idle,
                            compose_state_updated: Default::default(),
                        }
                    )])
                })
            }
        )
    }
}
