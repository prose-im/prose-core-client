// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::FullJid;

use crate::domain::rooms::models::{ComposeState, RoomAffiliation};
use crate::domain::shared::models::{OccupantId, UserResourceId};
use crate::dtos::{Availability, RoomId};

#[derive(Debug, PartialEq)]
pub enum ServerEvent {
    Room(RoomEvent),
    Placeholder,
}

#[derive(Debug, PartialEq)]
pub struct RoomEvent {
    pub room_id: RoomId,
    pub r#type: RoomEventType,
}

#[derive(Debug, PartialEq)]
pub struct RoomUserInfo {
    pub id: OccupantId,
    pub real_id: Option<UserResourceId>,
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
        alternate_room: Option<RoomId>,
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
