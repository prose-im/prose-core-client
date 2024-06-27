// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;

use crate::app::deps::DynMessagesRepository;
use crate::domain::messaging::models::MessageLikePayload;
use crate::domain::rooms::models::{
    ParticipantList, RegisteredMember, RoomFeatures, RoomSessionParticipant,
};
use crate::domain::settings::models::SyncedRoomSettings;
use crate::domain::shared::models::{AccountId, RoomId, RoomType, UserId};
use crate::domain::sidebar::models::Bookmark;
use crate::domain::user_info::models::Presence;
use crate::dtos::{OccupantId, ParticipantId};

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

impl RoomState {
    pub fn is_disconnected(&self) -> bool {
        let Self::Disconnected { .. } = self else {
            return false;
        };
        return true;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoomInfo {
    /// The JID of the room.
    pub room_id: RoomId,
    /// The nickname with which our user is connected to the room.
    pub user_nickname: String,
    /// The type of the room.
    pub r#type: RoomType,
    /// The room's supported features.
    pub features: RoomFeatures,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RoomStatistics {
    /// Are the statistics out of date?
    needs_update: bool,
    /// The number of unread messages in this room.
    pub unread_count: u32,
    /// The number of unread messages mentioning our user in this room.
    pub mentions_count: u32,
}

impl Default for RoomStatistics {
    fn default() -> Self {
        Self {
            needs_update: true,
            unread_count: 0,
            mentions_count: 0,
        }
    }
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
    /// Some tidbits about this room.
    pub statistics: RoomStatistics,
    /// The room's settings
    pub settings: SyncedRoomSettings,
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

    pub fn with_participants<T>(&self, f: impl FnOnce(&ParticipantList) -> T) -> T {
        f(&self.inner.details.read().participants)
    }

    pub fn with_participants_mut<T>(&self, f: impl FnOnce(&mut ParticipantList) -> T) -> T {
        f(&mut self.inner.details.write().participants)
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

    pub fn statistics(&self) -> RoomStatistics {
        self.inner.details.read().statistics.clone()
    }

    pub fn set_needs_update_statistics(&self) {
        self.inner.details.write().statistics.needs_update = true;
    }

    pub async fn update_statistics_if_needed(
        &self,
        account: &AccountId,
        messages_repo: &DynMessagesRepository,
    ) -> Result<RoomStatistics> {
        match self.state() {
            RoomState::Pending | RoomState::Connecting => return Ok(Default::default()),
            RoomState::Connected | RoomState::Disconnected { .. } => (),
        }

        let last_read_message = {
            let guard = self.inner.details.read();
            if !guard.statistics.needs_update {
                return Ok(guard.statistics.clone());
            }
            guard.settings.last_read_message.clone()
        };

        let mut stats = RoomStatistics::default();
        stats.needs_update = false;

        self.inner.details.write().statistics = stats.clone();

        let last_read_message_timestamp = last_read_message
            .map(|message_ref| message_ref.timestamp)
            .unwrap_or(DateTime::<Utc>::MIN_UTC);

        let messages = messages_repo
            .get_messages_after(account, &self.room_id, last_read_message_timestamp)
            .await?;

        let our_user_id = ParticipantId::User(account.to_user_id());
        let our_participant_id = self.occupant_id().map(ParticipantId::Occupant);

        let is_muc = self.room_id.is_muc_room();

        for message in messages {
            let MessageLikePayload::Message { ref mentions, .. } = message.payload else {
                continue;
            };

            let is_our_message = if is_muc {
                // We're generally trying to resolve OccupantIDs into UserIDs if possible.
                // So the sender could be either/or depending on the room configuration.
                Some(&message.from) == our_participant_id.as_ref() || message.from == our_user_id
            } else {
                message.from == our_user_id
            };

            if is_our_message {
                continue;
            }

            for mention in mentions {
                if account == &mention.user {
                    stats.mentions_count += 1;
                    break;
                }
            }

            stats.unread_count += 1;
        }

        self.inner.details.write().statistics = stats.clone();
        Ok(stats)
    }

    pub fn settings(&self) -> SyncedRoomSettings {
        self.inner.details.read().settings.clone()
    }

    pub fn with_settings_mut<T>(&self, f: impl FnOnce(&mut SyncedRoomSettings) -> T) -> T {
        f(&mut self.inner.details.write().settings)
    }
}

impl Room {
    pub fn pending(bookmark: &Bookmark, nickname: &str) -> Self {
        let participants = match &bookmark.jid {
            RoomId::User(user_id) => ParticipantList::for_direct_message(
                user_id,
                user_id.username(),
                Presence::default(),
            ),
            RoomId::Muc(_) => Default::default(),
        };

        Self::new(
            RoomInfo {
                room_id: bookmark.jid.clone(),
                user_nickname: nickname.to_string(),
                r#type: bookmark.r#type.into(),
                features: Default::default(),
            },
            RoomDetails {
                name: Some(bookmark.name.clone()),
                description: None,
                topic: None,
                participants,
                sidebar_state: bookmark.sidebar_state,
                state: RoomState::Pending,
                statistics: Default::default(),
                settings: SyncedRoomSettings::new(bookmark.jid.clone()),
            },
        )
    }

    pub fn connecting(room_id: &RoomId, nickname: &str, sidebar_state: RoomSidebarState) -> Self {
        Self::new(
            RoomInfo {
                room_id: room_id.clone(),
                user_nickname: nickname.to_string(),
                r#type: RoomType::Unknown,
                features: Default::default(),
            },
            RoomDetails {
                name: None,
                description: None,
                topic: None,
                participants: Default::default(),
                sidebar_state,
                state: RoomState::Connecting,
                statistics: Default::default(),
                settings: SyncedRoomSettings::new(room_id.clone()),
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
        settings: SyncedRoomSettings,
    ) -> Self {
        assert!(self.is_connecting(), "Cannot promote a non-connecting room");

        let mut details = self.inner.details.read().clone();
        details.name = name;
        details.description = description;
        details.topic = topic;
        details.participants = ParticipantList::new(members, participants);
        details.state = RoomState::Connected;
        details.settings = settings;

        Self::new(info, details)
    }

    pub fn by_changing_type(&self, new_type: RoomType) -> Self {
        Self::new(
            RoomInfo {
                room_id: self.room_id.clone(),
                user_nickname: self.user_nickname.clone(),
                r#type: new_type,
                features: self.features.clone(),
            },
            self.inner.details.read().clone(),
        )
    }
}

impl Room {
    pub fn for_direct_message(
        contact_id: &UserId,
        contact_name: &str,
        presence: Presence,
        sidebar_state: RoomSidebarState,
        features: RoomFeatures,
        settings: SyncedRoomSettings,
    ) -> Self {
        Self::new(
            RoomInfo {
                room_id: RoomId::from(contact_id.clone()),
                user_nickname: "no_nickname".to_string(),
                r#type: RoomType::DirectMessage,
                features,
            },
            RoomDetails {
                name: Some(contact_name.to_string()),
                description: None,
                topic: None,
                participants: ParticipantList::for_direct_message(
                    contact_id,
                    contact_name,
                    presence,
                ),
                sidebar_state,
                state: RoomState::Connected,
                statistics: Default::default(),
                settings,
            },
        )
    }
}

#[cfg(feature = "test")]
impl Room {
    pub fn mock(info: RoomInfo) -> Self {
        let room_id = info.room_id.clone();
        Self::new(
            info,
            RoomDetails {
                name: None,
                description: None,
                topic: None,
                participants: Default::default(),
                sidebar_state: RoomSidebarState::InSidebar,
                state: Default::default(),
                statistics: Default::default(),
                settings: SyncedRoomSettings::new(room_id),
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
    use crate::domain::shared::models::Availability;
    use crate::user_id;

    use super::*;

    #[test]
    fn test_room_internals_for_direct_message() {
        let internals = Room::for_direct_message(
            &user_id!("contact@prose.org"),
            "Jane Doe",
            Presence {
                availability: Availability::Available,
                ..Default::default()
            },
            RoomSidebarState::Favorite,
            Default::default(),
            SyncedRoomSettings::new(user_id!("contact@prose.org").into()),
        );

        assert_eq!(
            internals,
            Room::new(
                RoomInfo {
                    room_id: user_id!("contact@prose.org").into(),
                    user_nickname: "no_nickname".to_string(),
                    r#type: RoomType::DirectMessage,
                    features: Default::default(),
                },
                RoomDetails {
                    name: Some("Jane Doe".to_string()),
                    description: None,
                    topic: None,
                    participants: ParticipantList::for_direct_message(
                        &user_id!("contact@prose.org"),
                        "Jane Doe",
                        Presence {
                            availability: Availability::Available,
                            ..Default::default()
                        },
                    ),
                    sidebar_state: RoomSidebarState::Favorite,
                    state: RoomState::Connected,
                    statistics: Default::default(),
                    settings: SyncedRoomSettings::new(user_id!("contact@prose.org").into()),
                }
            )
        )
    }
}
