// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::presence;

use crate::app::dtos::Availability;

impl From<(Option<presence::Type>, Option<presence::Show>)> for Availability {
    fn from(value: (Option<presence::Type>, Option<presence::Show>)) -> Self {
        // https://datatracker.ietf.org/doc/html/rfc6121#section-4.7.1
        match (value.0, value.1) {
            // The absence of a 'type' attribute signals that the relevant entity is
            // available for communication (see Section 4.2 and Section 4.4).
            (None, None) => Availability::Available,
            (None, Some(presence::Show::Away)) => Availability::Away,
            (None, Some(presence::Show::Chat)) => Availability::Available,
            (None, Some(presence::Show::Dnd)) => Availability::DoNotDisturb,
            (None, Some(presence::Show::Xa)) => Availability::Away,
            (Some(_), _) => Availability::Unavailable,
        }
    }
}

impl TryFrom<Availability> for presence::Show {
    type Error = anyhow::Error;

    fn try_from(value: Availability) -> Result<Self, Self::Error> {
        match value {
            Availability::Available => Ok(presence::Show::Chat),
            Availability::Unavailable => Err(anyhow::format_err!(
                "You cannot set yourself to Unavailable. Choose 'Away' instead."
            )),
            Availability::DoNotDisturb => Ok(presence::Show::Dnd),
            Availability::Away => Ok(presence::Show::Away),
        }
    }
}
