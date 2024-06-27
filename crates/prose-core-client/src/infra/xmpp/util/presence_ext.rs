// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::presence;
use xmpp_parsers::presence::Presence;
use xmpp_parsers::vcard_update::VCardUpdate;

use prose_xmpp::ns;

use crate::domain::shared::models::{AnonOccupantId, Availability, AvatarId};

pub trait PresenceExt {
    fn anon_occupant_id(&self) -> Option<AnonOccupantId>;
    fn availability(&self) -> Availability;
    fn avatar_id(&self) -> Option<AvatarId>;
}

impl PresenceExt for Presence {
    fn anon_occupant_id(&self) -> Option<AnonOccupantId> {
        self.payloads
            .iter()
            .find(|p| p.is("occupant-id", ns::OCCUPANT_ID))
            .and_then(|e| e.attr("id"))
            .map(|id| AnonOccupantId::from(id.to_string()))
    }

    fn availability(&self) -> Availability {
        Availability::from((
            (self.type_ != presence::Type::None).then_some(self.type_.clone()),
            self.show.clone(),
        ))
    }

    fn avatar_id(&self) -> Option<AvatarId> {
        self.payloads
            .iter()
            .find(|p| p.is("x", ns::VCARD_UPDATE))
            .cloned()
            .and_then(|p| VCardUpdate::try_from(p).ok())
            .and_then(|vcard| vcard.photo)
            .and_then(|photo| photo.data)
            .map(|sha1_bytes| {
                let mut sha1_str = String::with_capacity(40);
                for byte in sha1_bytes {
                    sha1_str.extend(format!("{:02x}", byte).chars());
                }
                AvatarId::from_str_unchecked(sha1_str)
            })
    }
}
