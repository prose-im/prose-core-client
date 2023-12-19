// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0

use xmpp_parsers::muc::user::Affiliation;

use crate::domain::rooms::models::RoomAffiliation;

impl From<Affiliation> for RoomAffiliation {
    fn from(value: Affiliation) -> Self {
        match value {
            Affiliation::Owner => RoomAffiliation::Owner,
            Affiliation::Admin => RoomAffiliation::Admin,
            Affiliation::Member => RoomAffiliation::Member,
            Affiliation::Outcast => RoomAffiliation::Outcast,
            Affiliation::None => RoomAffiliation::None,
        }
    }
}
