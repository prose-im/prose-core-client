// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::rooms::{Group, PrivateChannel, PublicChannel};
use crate::types::muc::rooms::{AbstractRoom, GenericRoom};
use crate::types::muc::RoomMetadata;
use jid::BareJid;
use std::cmp::Ordering;
use xmpp_parsers::presence::Presence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Room {
    Group(Group),
    PrivateChannel(PrivateChannel),
    PublicChannel(PublicChannel),
    /// A generic MUC room that doesn't match any of our requirements
    Generic(GenericRoom),
}

impl Room {
    pub fn jid(&self) -> &BareJid {
        match self {
            Room::Group(room) => &room.room.jid,
            Room::PrivateChannel(room) => &room.room.jid,
            Room::PublicChannel(room) => &room.room.jid,
            Room::Generic(room) => &room.room.jid,
        }
    }

    pub fn nick(&self) -> &str {
        &self.abstract_room().nick
    }

    pub fn name(&self) -> Option<&str> {
        self.abstract_room().name.as_deref()
    }

    pub fn handle_presence(&mut self, presence: Presence) {
        println!("RECEIVED PRESENCE: {:?}", presence);
    }
}

impl Room {
    fn abstract_room(&self) -> &AbstractRoom {
        match self {
            Room::Group(room) => &room.room,
            Room::PrivateChannel(room) => &room.room,
            Room::PublicChannel(room) => &room.room,
            Room::Generic(room) => &room.room,
        }
    }

    fn sort_value(&self) -> i32 {
        match self {
            Room::Group(_) => 0,
            Room::PrivateChannel(_) => 1,
            Room::PublicChannel(_) => 2,
            Room::Generic(_) => 3,
        }
    }
}

impl PartialOrd for Room {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Room {
    fn cmp(&self, other: &Self) -> Ordering {
        let sort_val1 = self.sort_value();
        let sort_val2 = other.sort_value();

        if sort_val1 < sort_val2 {
            return Ordering::Less;
        } else if sort_val1 > sort_val2 {
            return Ordering::Greater;
        }

        self.name()
            .unwrap_or_default()
            .cmp(other.name().unwrap_or_default())
    }
}

impl From<RoomMetadata> for Room {
    fn from(value: RoomMetadata) -> Self {
        let room = AbstractRoom {
            jid: value.room_jid.to_bare(),
            nick: value.room_jid.resource_str().to_string(),
            name: value.settings.name,
            description: value.settings.description,
            occupants: vec![],
        };

        let features = value.settings.features;

        match features {
            _ if features.can_act_as_group() => Room::Group(Group { room }),
            _ if features.can_act_as_private_channel() => {
                Room::PrivateChannel(PrivateChannel { room })
            }
            _ if features.can_act_as_public_channel() => {
                Room::PublicChannel(PublicChannel { room })
            }
            _ => Room::Generic(GenericRoom { room }),
        }
    }
}
