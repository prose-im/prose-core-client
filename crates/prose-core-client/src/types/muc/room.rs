// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::rooms::{Group, PendingRoom, PrivateChannel, PublicChannel};
use crate::types::muc::rooms::{AbstractRoom, GenericRoom};
use jid::BareJid;
use xmpp_parsers::presence::Presence;

#[derive(Debug, Clone)]
pub enum Room {
    /// A room that is being entered and that might still be missing information.
    Pending(PendingRoom),
    Group(Group),
    PrivateChannel(PrivateChannel),
    PublicChannel(PublicChannel),
    /// A generic MUC room that doesn't match any of our requirements
    Generic(GenericRoom),
}

impl Room {
    pub fn pending(jid: &BareJid) -> Self {
        Room::Pending(PendingRoom::new(jid))
    }
}

impl Room {
    pub fn jid(&self) -> &BareJid {
        match self {
            Room::Pending(room) => &room.jid,
            Room::Group(room) => &room.room.jid,
            Room::PrivateChannel(room) => &room.room.jid,
            Room::PublicChannel(room) => &room.room.jid,
            Room::Generic(room) => &room.room.jid,
        }
    }

    pub fn handle_presence(&mut self, presence: Presence) {
        println!("RECEIVED PRESENCE: {:?}", presence);
    }
}

impl Room {
    fn abstract_room(&self) -> Option<&AbstractRoom> {
        match self {
            Room::Pending(_) => None,
            Room::Group(room) => Some(&room.room),
            Room::PrivateChannel(room) => Some(&room.room),
            Room::PublicChannel(room) => Some(&room.room),
            Room::Generic(room) => Some(&room.room),
        }
    }
}
