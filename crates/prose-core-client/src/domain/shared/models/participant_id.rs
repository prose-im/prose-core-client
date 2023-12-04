// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{OccupantId, UserId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Represents the identifier of a user within - what we define as - room. So it could be either a
/// regular UserId (BareJid) in a DirectMessage room (1:1 conversation) or a OccupantId when in a
/// multi-user room (MUC chat).
pub enum ParticipantId {
    User(UserId),
    Occupant(OccupantId),
}

impl From<UserId> for ParticipantId {
    fn from(value: UserId) -> Self {
        ParticipantId::User(value)
    }
}

impl From<OccupantId> for ParticipantId {
    fn from(value: OccupantId) -> Self {
        ParticipantId::Occupant(value)
    }
}
