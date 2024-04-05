// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;
use std::sync::Arc;

use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

use crate::domain::rooms::models::{ParticipantList, RegisteredMember, RoomSessionParticipant};
use crate::domain::shared::models::{Availability, RoomId, RoomType, UserId};
use crate::domain::sidebar::models::Bookmark;
use crate::dtos::OccupantId;

/// Contains information about a connected room and its state.
#[derive(Debug, Clone)]
pub struct Room {
    inner: Arc<RoomInner>,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum RoomSidebarState {
    /// The room is not visible in the sidebar.
    NotInSidebar,
    /// The room is visible in the sidebar.
    InSidebar,
    /// The room is visible in the sidebar as a favorite.
    Favorite,
}

impl RoomSidebarState {
    pub fn is_in_sidebar(&self) -> bool {
        match self {
            Self::NotInSidebar => false,
            Self::InSidebar | Self::Favorite => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum RoomState {
    /// The room has been inserted from a bookmark and is waiting to be connected.
    #[default]
    Pending,
    Connecting,
    Connected,
    Disconnected {
        error: Option<String>,
        can_retry: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoomInfo {
    /// The JID of the room.
    pub room_id: RoomId,
    /// The nickname with which our user is connected to the room.
    pub user_nickname: String,
    /// The type of the room.
    pub r#type: RoomType,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RoomDetails {
    /// The name of the room.
    pub name: Option<String>,
    /// The description of the room.
    pub description: Option<String>,
    /// The room's topic.
    pub topic: Option<String>,
    /// The participants in the room.
    pub participants: ParticipantList,
    /// Whether the room is visible in the sidebar.
    pub sidebar_state: RoomSidebarState,
    /// The state the room is in.
    pub state: RoomState,
    /// The number of unread messages in this room.
    pub unread_count: u32,
    /// The number of unread messages mentioning our user in this room.
    pub mentions_count: u32,
    /// Should messages be OMEMO encrypted?
    pub encryption_enabled: bool,
}

#[derive(Debug)]
struct RoomInner {
    info: RoomInfo,
    details: RwLock<RoomDetails>,
}

impl Deref for Room {
    type Target = RoomInfo;

    fn deref(&self) -> &Self::Target {
        &self.inner.info
    }
}

impl Room {
    fn new(info: RoomInfo, details: RoomDetails) -> Self {
        Self {
            inner: Arc::new(RoomInner {
                info,
                details: RwLock::new(details),
            }),
        }
    }
}

impl Room {
    pub fn name(&self) -> Option<String> {
        self.inner.details.read().name.clone()
    }

    pub fn set_name(&self, name: Option<String>) {
        self.inner.details.write().name = name
    }

    pub fn description(&self) -> Option<String> {
        self.inner.details.read().description.clone()
    }

    pub fn set_description(&self, name: Option<String>) {
        self.inner.details.write().description = name
    }

    pub fn topic(&self) -> Option<String> {
        self.inner.details.read().topic.clone()
    }

    pub fn set_topic(&self, topic: Option<String>) {
        self.inner.details.write().topic = topic
    }

    pub fn participants(&self) -> MappedRwLockReadGuard<ParticipantList> {
        RwLockReadGuard::map(self.inner.details.read(), |s| &s.participants)
    }

    pub fn participants_mut(&self) -> MappedRwLockWriteGuard<ParticipantList> {
        RwLockWriteGuard::map(self.inner.details.write(), |s| &mut s.participants)
    }

    pub fn sidebar_state(&self) -> RoomSidebarState {
        self.inner.details.read().sidebar_state
    }

    pub fn set_sidebar_state(&self, state: RoomSidebarState) {
        self.inner.details.write().sidebar_state = state
    }

    pub fn state(&self) -> RoomState {
        self.inner.details.read().state.clone()
    }

    pub fn set_state(&self, state: RoomState) {
        self.inner.details.write().state = state
    }

    pub fn unread_count(&self) -> u32 {
        self.inner.details.read().unread_count
    }

    pub fn mentions_count(&self) -> u32 {
        self.inner.details.read().mentions_count
    }

    pub fn encryption_enabled(&self) -> bool {
        self.inner.details.read().encryption_enabled
    }

    pub fn set_encryption_enabled(&self, enabled: bool) {
        self.inner.details.write().encryption_enabled = enabled
    }

    pub fn increment_unread_count(&self) {
        let mut guard = self.inner.details.write();
        guard.unread_count = guard.unread_count.saturating_add(1);
    }

    pub fn increment_mentions_count(&self) {
        let mut guard = self.inner.details.write();
        guard.mentions_count = guard.mentions_count.saturating_add(1);
    }

    pub fn mark_as_read(&self) {
        let mut guard = self.inner.details.write();
        guard.unread_count = 0;
        guard.mentions_count = 0;
    }
}

impl Room {
    pub fn pending(bookmark: &Bookmark, nickname: &str) -> Self {
        let participants = match &bookmark.jid {
            RoomId::User(user_id) => ParticipantList::for_direct_message(
                user_id,
                user_id.username(),
                Availability::Unavailable,
            ),
            RoomId::Muc(_) => Default::default(),
        };

        Self::new(
            RoomInfo {
                room_id: bookmark.jid.clone(),
                user_nickname: nickname.to_string(),
                r#type: bookmark.r#type.into(),
            },
            RoomDetails {
                name: Some(bookmark.name.clone()),
                description: None,
                topic: None,
                participants,
                sidebar_state: bookmark.sidebar_state,
                state: RoomState::Pending,
                unread_count: 0,
                mentions_count: 0,
                encryption_enabled: false,
            },
        )
    }

    pub fn connecting(room_id: &RoomId, nickname: &str, sidebar_state: RoomSidebarState) -> Self {
        Self::new(
            RoomInfo {
                room_id: room_id.clone(),
                user_nickname: nickname.to_string(),
                r#type: RoomType::Unknown,
            },
            RoomDetails {
                name: None,
                description: None,
                topic: None,
                participants: Default::default(),
                sidebar_state,
                state: RoomState::Connecting,
                unread_count: 0,
                mentions_count: 0,
                encryption_enabled: false,
            },
        )
    }

    pub fn is_connecting(&self) -> bool {
        self.inner.details.read().state == RoomState::Connecting
    }
    pub fn is_pending(&self) -> bool {
        self.inner.details.read().state == RoomState::Pending
    }

    // Resolves a pending room.
    pub fn by_resolving_with_info(
        &self,
        name: Option<String>,
        description: Option<String>,
        topic: Option<String>,
        info: RoomInfo,
        members: Vec<RegisteredMember>,
        participants: Vec<RoomSessionParticipant>,
    ) -> Self {
        assert!(self.is_connecting(), "Cannot promote a non-connecting room");

        let mut details = self.inner.details.read().clone();
        details.name = name;
        details.description = description;
        details.topic = topic;
        details.participants = ParticipantList::new(members, participants);
        details.state = RoomState::Connected;

        Self::new(info, details)
    }

    pub fn by_changing_type(&self, new_type: RoomType) -> Self {
        Self::new(
            RoomInfo {
                room_id: self.room_id.clone(),
                user_nickname: self.user_nickname.clone(),
                r#type: new_type,
            },
            self.inner.details.read().clone(),
        )
    }
}

impl Room {
    pub fn for_direct_message(
        contact_id: &UserId,
        contact_name: &str,
        availability: Availability,
        sidebar_state: RoomSidebarState,
    ) -> Self {
        Self::new(
            RoomInfo {
                room_id: RoomId::from(contact_id.clone()),
                user_nickname: "no_nickname".to_string(),
                r#type: RoomType::DirectMessage,
            },
            RoomDetails {
                name: Some(contact_name.to_string()),
                description: None,
                topic: None,
                participants: ParticipantList::for_direct_message(
                    contact_id,
                    contact_name,
                    availability,
                ),
                sidebar_state,
                state: RoomState::Connected,
                unread_count: 0,
                mentions_count: 0,
                encryption_enabled: false,
            },
        )
    }
}

#[cfg(feature = "test")]
impl Room {
    pub fn mock(info: RoomInfo) -> Self {
        Self::new(
            info,
            RoomDetails {
                name: None,
                description: None,
                topic: None,
                participants: Default::default(),
                sidebar_state: RoomSidebarState::InSidebar,
                state: Default::default(),
                unread_count: 0,
                mentions_count: 0,
                encryption_enabled: false,
            },
        )
    }

    pub fn with_user_nickname(self, nickname: impl Into<String>) -> Self {
        let mut info = self.inner.info.clone();
        info.user_nickname = nickname.into();
        Self::new(info, self.inner.details.read().clone())
    }
}

impl RoomInfo {
    /// Returns the OccupantId of the connected user by appending their nickname to the room's
    /// bare jid.
    pub fn occupant_id(&self) -> Option<OccupantId> {
        match &self.room_id {
            RoomId::User(_) => None,
            RoomId::Muc(id) => Some(id.occupant_id_with_nickname(&self.user_nickname)
                .expect("The provided JID and user_nickname were invalid and could not be used to form a FullJid."))
        }
    }
}

#[cfg(feature = "test")]
impl PartialEq for Room {
    fn eq(&self, other: &Self) -> bool {
        self.inner.info == other.inner.info
            && *self.inner.details.read() == *other.inner.details.read()
    }
}

#[cfg(test)]
mod tests {
    use crate::user_id;

    use super::*;

    #[test]
    fn test_room_internals_for_direct_message() {
        let internals = Room::for_direct_message(
            &user_id!("contact@prose.org"),
            "Jane Doe",
            Availability::Available,
            RoomSidebarState::Favorite,
        );

        assert_eq!(
            internals,
            Room::new(
                RoomInfo {
                    room_id: user_id!("contact@prose.org").into(),
                    user_nickname: "no_nickname".to_string(),
                    r#type: RoomType::DirectMessage,
                },
                RoomDetails {
                    name: Some("Jane Doe".to_string()),
                    description: None,
                    topic: None,
                    participants: ParticipantList::for_direct_message(
                        &user_id!("contact@prose.org"),
                        "Jane Doe",
                        Availability::Available
                    ),
                    sidebar_state: RoomSidebarState::Favorite,
                    state: RoomState::Connected,
                    unread_count: 0,
                    mentions_count: 0,
                    encryption_enabled: false,
                }
            )
        )
    }
}
