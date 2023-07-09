use microtype::microtype;
use xmpp_parsers::presence;

microtype! {
    pub prose_core_domain::Availability {
        Availability
    }
}

impl From<(Option<presence::Type>, Option<presence::Show>)> for Availability {
    fn from(value: (Option<presence::Type>, Option<presence::Show>)) -> Self {
        use prose_core_domain::Availability;

        // https://datatracker.ietf.org/doc/html/rfc6121#section-4.7.1
        Availability(match (value.0, value.1) {
            // The absence of a 'type' attribute signals that the relevant entity is
            // available for communication (see Section 4.2 and Section 4.4).
            (None, None) => Availability::Available,
            (None, Some(presence::Show::Away)) => Availability::Away,
            (None, Some(presence::Show::Chat)) => Availability::Available,
            (None, Some(presence::Show::Dnd)) => Availability::DoNotDisturb,
            (None, Some(presence::Show::Xa)) => Availability::Away,
            (Some(_), _) => Availability::Unavailable,
        })
    }
}

impl TryFrom<Availability> for xmpp_parsers::presence::Show {
    type Error = anyhow::Error;

    fn try_from(value: Availability) -> Result<Self, Self::Error> {
        use prose_core_domain::Availability;

        match value.0 {
            Availability::Available => Ok(presence::Show::Chat),
            Availability::Unavailable => Err(anyhow::format_err!(
                "You cannot set yourself to Unavailable. Choose 'Away' instead."
            )),
            Availability::DoNotDisturb => Ok(presence::Show::Dnd),
            Availability::Away => Ok(presence::Show::Away),
        }
    }
}
