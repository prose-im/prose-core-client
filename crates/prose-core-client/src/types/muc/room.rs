// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::rooms::{Group, PrivateChannel, PublicChannel};
use crate::types::muc::rooms::{AbstractRoom, GenericRoom};
use crate::types::muc::RoomMetadata;
use jid::BareJid;
use xmpp_parsers::presence::Presence;

#[derive(Debug, Clone)]
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
