// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;

use crate::types::muc::{Room, RoomInfo};
pub(self) use abstract_room::{AbstractRoom, Occupant};
use prose_xmpp::Client as XMPPClient;

mod abstract_room;

#[derive(Debug)]
pub struct Group {
    room: AbstractRoom,
}

#[derive(Debug)]
pub struct PrivateChannel {
    room: AbstractRoom,
}

#[derive(Debug)]
pub struct PublicChannel {
    room: AbstractRoom,
}

#[derive(Debug)]
pub struct GenericRoom {
    room: AbstractRoom,
}

#[derive(Debug)]
pub struct PendingRoom {
    jid: BareJid,
    occupants: Vec<Occupant>,
}

impl PendingRoom {
    pub fn new(jid: &BareJid) -> Self {
        PendingRoom {
            jid: jid.clone(),
            occupants: vec![],
        }
    }

    pub fn into_room(self, info: &RoomInfo, client: XMPPClient) -> Room {
        let room = AbstractRoom {
            jid: self.jid,
            name: info.name.clone(),
            description: info.description.clone(),
            client,
            occupants: self.occupants,
        };

        match info {
            _ if info.features.can_act_as_group() => Room::Group(Group { room }),
            _ if info.features.can_act_as_private_channel() => {
                Room::PrivateChannel(PrivateChannel { room })
            }
            _ if info.features.can_act_as_public_channel() => {
                Room::PublicChannel(PublicChannel { room })
            }
            _ => Room::Generic(GenericRoom { room }),
        }
    }
}
