// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::presence;
use xmpp_parsers::presence::Presence;

use prose_xmpp::ns;

use crate::domain::shared::models::AnonOccupantId;
use crate::dtos::Availability;

pub trait PresenceExt {
    fn anon_occupant_id(&self) -> Option<AnonOccupantId>;
    fn availability(&self) -> Availability;
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
}
