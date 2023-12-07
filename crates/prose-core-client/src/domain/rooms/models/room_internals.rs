// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::{Deref, DerefMut};

use jid::FullJid;
use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

use crate::domain::rooms::models::{ParticipantList, RegisteredMember};
use crate::domain::shared::models::{RoomId, RoomType, UserId};

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

#[derive(Clone, Default, Debug, PartialEq)]
pub struct RoomState {
    /// The name of the room.
    pub name: Option<String>,
    /// The room's topic.
    pub topic: Option<String>,
    /// The participants in the room.
    pub participants: ParticipantList,
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

    pub fn participants(&self) -> MappedRwLockReadGuard<ParticipantList> {
        RwLockReadGuard::map(self.state.read(), |s| &s.participants)
    }

    pub fn participants_mut(&self) -> MappedRwLockWriteGuard<ParticipantList> {
        RwLockWriteGuard::map(self.state.write(), |s| &mut s.participants)
    }
}

impl RoomInternals {
    pub fn pending(room_id: &RoomId, nickname: &str) -> Self {
        Self {
            info: RoomInfo {
                room_id: room_id.clone(),
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
        members: Vec<RegisteredMember>,
    ) -> Self {
        assert!(self.is_pending(), "Cannot promote a non-pending room");

        let mut state = self.state.read().clone();
        state.name = name;
        state.participants.set_registered_members(members);

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
                participants: ParticipantList::for_direct_message(contact_id, contact_name),
            }),
        }
    }
}

#[cfg(feature = "test")]
impl RoomInternals {
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
    use crate::{room_id, user_id};

    use super::*;

    #[test]
    fn test_room_internals_for_direct_message() {
        let internals =
            RoomInternals::for_direct_message(&user_id!("contact@prose.org"), "Jane Doe");

        assert_eq!(
            internals,
            RoomInternals {
                info: RoomInfo {
                    room_id: room_id!("contact@prose.org"),
                    description: None,
                    user_nickname: "no_nickname".to_string(),
                    r#type: RoomType::DirectMessage,
                },
                state: RwLock::new(RoomState {
                    name: Some("Jane Doe".to_string()),
                    topic: None,
                    participants: ParticipantList::for_direct_message(
                        &user_id!("contact@prose.org"),
                        "Jane Doe"
                    )
                })
            }
        )
    }
}
