// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::presence::Presence;

use prose_xmpp::ns;

use crate::domain::shared::models::AnonOccupantId;

pub trait PresenceExt {
    fn anon_occupant_id(&self) -> Option<AnonOccupantId>;
}

impl PresenceExt for Presence {
    fn anon_occupant_id(&self) -> Option<AnonOccupantId> {
        self.payloads
            .iter()
            .find(|p| p.is("occupant-id", ns::OCCUPANT_ID))
            .and_then(|e| e.attr("id"))
            .map(|id| AnonOccupantId::from(id.to_string()))
    }
}
