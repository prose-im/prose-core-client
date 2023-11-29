// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::FullJid;

use crate::domain::rooms::models::{ComposeState, RoomAffiliation};
use crate::dtos::{Availability, RoomJid};

#[derive(Debug, PartialEq)]
pub enum ServerEvent {
    Room(RoomEvent),
    Placeholder,
}

#[derive(Debug, PartialEq)]
pub struct RoomEvent {
    pub room: RoomJid,
    pub r#type: RoomEventType,
}

#[derive(Debug, PartialEq)]
pub struct RoomUserInfo {
    pub jid: FullJid,
    pub real_jid: Option<FullJid>,
    pub affiliation: RoomAffiliation,
    pub availability: Availability,
    /// Is this the current (logged-in) user?
    pub is_self: bool,
}

#[derive(Debug, PartialEq)]
pub enum RoomEventType {
    UserAvailabilityOrMembershipChanged {
        user: RoomUserInfo,
    },
    UserWasDisconnectedByServer {
        user: RoomUserInfo,
    },
    UserWasPermanentlyRemoved {
        user: RoomUserInfo,
    },

    UserComposeStateChanged {
        user_id: FullJid,
        state: ComposeState,
    },

    RoomWasDestroyed {
        alternate_room: Option<RoomJid>,
    },
    /// The configuration _or_ name of the room was changed.
    RoomConfigChanged,
    RoomTopicChanged {
        new_topic: Option<String>,
    },

    ReceivedInvite {
        password: Option<String>,
    },
}
