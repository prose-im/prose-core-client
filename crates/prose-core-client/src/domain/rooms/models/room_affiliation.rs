// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

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
