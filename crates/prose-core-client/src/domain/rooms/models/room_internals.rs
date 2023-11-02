// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::{BareJid, Jid};
use parking_lot::RwLock;
use std::collections::HashMap;
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::muc::user::Affiliation;

use crate::domain::contacts::models::Contact;
use crate::domain::rooms::models::{RoomMetadata, RoomState};
use crate::domain::shared::models::RoomType;
use crate::dtos::Occupant;

/// Contains information about a connected room and its state.
#[derive(Debug)]
pub struct RoomInternals {
    pub info: RoomInfo,
    pub state: RwLock<RoomState>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoomInfo {
    /// The JID of the room.
    pub jid: BareJid,
    /// The name of the room.
    pub name: Option<String>,
    /// The description of the room.
    pub description: Option<String>,
    /// The JID of our logged-in user.
    pub user_jid: BareJid,
    /// The nickname with which our user is connected to the room.
    pub user_nickname: String,
    /// The list of members. Only available for DirectMessage and Group (member-only rooms).
    pub members: Vec<BareJid>,
    /// The type of the room.
    pub room_type: RoomType,
}

impl RoomInternals {
    pub fn pending(room_jid: &BareJid, user_jid: &BareJid, nickname: &str) -> Self {
        Self {
            info: RoomInfo {
                jid: room_jid.clone(),
                name: None,
                description: None,
                user_jid: user_jid.clone(),
                user_nickname: nickname.to_string(),
                members: vec![],
                room_type: RoomType::Pending,
            },
            state: Default::default(),
        }
    }

    pub fn is_pending(&self) -> bool {
        self.info.room_type == RoomType::Pending
    }

    pub fn to_permanent(&self, metadata: RoomMetadata) -> Self {
        assert!(self.is_pending(), "Cannot promote a non-pending room");

        let mut room = Self {
            info: self.info.clone(),
            state: RwLock::new(self.state.read().clone()),
        };

        room.info.jid = metadata.room_jid.to_bare();
        room.info.user_nickname = metadata.room_jid.resource_str().to_string();
        room.info.name = metadata.settings.name;
        room.info.description = metadata.settings.description;
        room.info.members = metadata.members;

        let features = &metadata.settings.features;

        room.info.room_type = match features {
            _ if features.can_act_as_group() => RoomType::Group,
            _ if features.can_act_as_private_channel() => RoomType::PrivateChannel,
            _ if features.can_act_as_public_channel() => RoomType::PublicChannel,
            _ => RoomType::Generic,
        };

        room
    }
}

impl RoomInternals {
    pub fn for_direct_message(user_jid: &BareJid, contact: &Contact, contact_name: &str) -> Self {
        Self {
            info: RoomInfo {
                jid: contact.jid.clone(),
                name: Some(contact_name.to_string()),
                description: None,
                user_jid: user_jid.clone(),
                user_nickname: "no_nickname".to_string(),
                members: vec![contact.jid.clone()],
                room_type: RoomType::DirectMessage,
            },
            state: RwLock::new(RoomState {
                subject: None,
                occupants: HashMap::from([(
                    Jid::Bare(contact.jid.clone()),
                    Occupant {
                        jid: Some(contact.jid.clone()),
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
impl PartialEq for RoomInternals {
    fn eq(&self, other: &Self) -> bool {
        self.info == other.info && *self.state.read() == *other.state.read()
    }
}

#[cfg(test)]
mod tests {
    use prose_xmpp::{bare, jid};
    use std::collections::HashMap;
    use xmpp_parsers::chatstates::ChatState;
    use xmpp_parsers::muc::user::Affiliation;

    use crate::domain::contacts::models::Group;
    use crate::dtos::Occupant;

    use super::*;

    #[test]
    fn test_room_internals_for_direct_message() {
        let internals = RoomInternals::for_direct_message(
            &bare!("logged-in-user@prose.org"),
            &Contact {
                jid: bare!("contact@prose.org"),
                name: None,
                group: Group::Favorite,
            },
            "Jane Doe",
        );

        assert_eq!(
            internals,
            RoomInternals {
                info: RoomInfo {
                    jid: bare!("contact@prose.org"),
                    name: Some("Jane Doe".to_string()),
                    description: None,
                    user_jid: bare!("logged-in-user@prose.org"),
                    user_nickname: "no_nickname".to_string(),
                    members: vec![bare!("contact@prose.org")],
                    room_type: RoomType::DirectMessage,
                },
                state: RwLock::new(RoomState {
                    subject: None,
                    occupants: HashMap::from([(
                        jid!("contact@prose.org"),
                        Occupant {
                            jid: Some(bare!("contact@prose.org")),
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
