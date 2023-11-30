// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Clone, Default)]
pub enum RoomAffiliation {
    /// The user who created the room, or who got appointed by its creator
    /// to be their equal.
    Owner,

    /// A user who has been empowered by an owner to do administrative
    /// operations.
    Admin,

    /// A user who is whitelisted to speak in moderated rooms, or to join a
    /// member-only room.
    Member,

    /// A user who has been banned from this room.
    Outcast,

    /// A normal participant.
    #[default]
    None,
}

impl Display for RoomAffiliation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RoomAffiliation::Owner => write!(f, "owner"),
            RoomAffiliation::Admin => write!(f, "admin"),
            RoomAffiliation::Member => write!(f, "member"),
            RoomAffiliation::Outcast => write!(f, "outcast"),
            RoomAffiliation::None => write!(f, "none"),
        }
    }
}
