// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use jid::{BareJid, FullJid, Jid};
use parking_lot::RwLock;
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::muc::user::Affiliation;

use crate::domain::rooms::models::RoomState;
use crate::domain::shared::models::{RoomJid, RoomType};
use crate::dtos::{Occupant, UserBasicInfo};

/// Contains information about a connected room and its state.
#[derive(Debug)]
pub struct RoomInternals {
    info: RoomInfo,
    state: RwLock<RoomState>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoomInfo {
    /// The JID of the room.
    pub jid: RoomJid,
    /// The description of the room.
    pub description: Option<String>,
    /// The JID of our logged-in user.
    pub user_jid: BareJid,
    /// The nickname with which our user is connected to the room.
    pub user_nickname: String,
    /// The list of members. Only available for DirectMessage and Group (member-only rooms).
    pub members: HashMap<BareJid, Member>,
    /// The type of the room.
    pub r#type: RoomType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Member {
    pub name: String,
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

    pub fn set_topic(&self, topic: Option<&str>) {
        self.state.write().topic = topic.map(ToString::to_string)
    }

    pub fn occupants(&self) -> Vec<Occupant> {
        self.state.read().occupants.values().cloned().collect()
    }

    pub fn get_occupant(&self, jid: &Jid) -> Option<Occupant> {
        self.state.read().occupants.get(&jid).cloned()
    }

    pub fn insert_occupant(
        &self,
        jid: &Jid,
        real_jid: Option<&BareJid>,
        name: Option<&str>,
        affiliation: &Affiliation,
    ) {
        self.state
            .write()
            .insert_occupant(jid, real_jid, name, affiliation)
    }

    pub fn remove_occupant(&self, jid: &Jid) {
        self.state.write().occupants.remove(jid);
    }

    pub fn set_occupant_chat_state(
        &self,
        occupant_jid: &Jid,
        timestamp: &DateTime<Utc>,
        chat_state: ChatState,
    ) {
        self.state
            .write()
            .set_occupant_chat_state(occupant_jid, timestamp, chat_state)
    }

    /// Returns the real JIDs of all composing users that started composing after `started_after`.
    /// If we don't have a real JID for a composing user they are excluded from the list.
    pub fn composing_users(&self, started_after: DateTime<Utc>) -> Vec<UserBasicInfo> {
        self.state.read().composing_users(started_after)
    }
}

impl RoomInternals {
    pub fn pending(room_jid: &RoomJid, user_jid: &BareJid, nickname: &str) -> Self {
        Self {
            info: RoomInfo {
                jid: room_jid.clone(),
                description: None,
                user_jid: user_jid.clone(),
                user_nickname: nickname.to_string(),
                members: HashMap::new(),
                r#type: RoomType::Pending,
            },
            state: Default::default(),
        }
    }

    pub fn is_pending(&self) -> bool {
        self.info.r#type == RoomType::Pending
    }

    // Resolves a pending room.
    pub fn by_resolving_with_info(&self, name: Option<String>, info: RoomInfo) -> Self {
        assert!(self.is_pending(), "Cannot promote a non-pending room");

        let mut state = self.state.read().clone();
        state.name = name;

        Self {
            info,
            state: RwLock::new(state),
        }
    }
}

impl RoomInternals {
    pub fn for_direct_message(
        user_jid: &BareJid,
        contact_jid: &BareJid,
        contact_name: &str,
    ) -> Self {
        Self {
            info: RoomInfo {
                jid: contact_jid.clone().into(),
                description: None,
                user_jid: user_jid.clone(),
                user_nickname: "no_nickname".to_string(),
                members: HashMap::from([(
                    contact_jid.clone(),
                    Member {
                        name: contact_name.to_string(),
                    },
                )]),
                r#type: RoomType::DirectMessage,
            },
            state: RwLock::new(RoomState {
                name: Some(contact_name.to_string()),
                topic: None,
                occupants: HashMap::from([(
                    Jid::Bare(contact_jid.clone()),
                    Occupant {
                        jid: Some(contact_jid.clone()),
                        name: Some(contact_name.to_string()),
                        affiliation: Affiliation::Owner,
                        chat_state: ChatState::Gone,
                        chat_state_updated: Default::default(),
                    },
                )]),
            }),
        }
    }
}

#[cfg(feature = "test")]
impl RoomInternals {
    pub fn set_occupants(&self, occupants: HashMap<Jid, Occupant>) {
        self.state.write().occupants = occupants;
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
        self.jid.with_resource_str(&self.user_nickname)
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

    use xmpp_parsers::chatstates::ChatState;
    use xmpp_parsers::muc::user::Affiliation;

    use prose_xmpp::{bare, jid};

    use crate::dtos::Occupant;

    use super::*;

    #[test]
    fn test_room_internals_for_direct_message() {
        let internals = RoomInternals::for_direct_message(
            &bare!("logged-in-user@prose.org"),
            &bare!("contact@prose.org"),
            "Jane Doe",
        );

        assert_eq!(
            internals,
            RoomInternals {
                info: RoomInfo {
                    jid: bare!("contact@prose.org").into(),
                    description: None,
                    user_jid: bare!("logged-in-user@prose.org"),
                    user_nickname: "no_nickname".to_string(),
                    members: HashMap::from([(
                        bare!("contact@prose.org"),
                        Member {
                            name: "Jane Doe".to_string()
                        }
                    )]),
                    r#type: RoomType::DirectMessage,
                },
                state: RwLock::new(RoomState {
                    name: Some("Jane Doe".to_string()),
                    topic: None,
                    occupants: HashMap::from([(
                        jid!("contact@prose.org"),
                        Occupant {
                            jid: Some(bare!("contact@prose.org")),
                            name: Some("Jane Doe".to_string()),
                            affiliation: Affiliation::Owner,
                            chat_state: ChatState::Gone,
                            chat_state_updated: Default::default(),
                        }
                    )])
                })
            }
        )
    }
}
