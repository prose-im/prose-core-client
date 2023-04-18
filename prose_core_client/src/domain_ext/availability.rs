use microtype::microtype;

use prose_core_lib::stanza::presence;

microtype! {
    pub prose_core_domain::Availability {
        Availability
    }
}

impl From<(Option<presence::Kind>, Option<presence::Show>)> for Availability {
    fn from(value: (Option<presence::Kind>, Option<presence::Show>)) -> Self {
        use prose_core_domain::Availability;

        // https://datatracker.ietf.org/doc/html/rfc6121#section-4.7.1
        Availability(match (value.0, value.1) {
            // The absence of a 'type' attribute signals that the relevant entity is
            // available for communication (see Section 4.2 and Section 4.4).
            (None, None) => Availability::Available,
            (None, Some(presence::Show::Away)) => Availability::Away,
            (None, Some(presence::Show::Chat)) => Availability::Available,
            (None, Some(presence::Show::DND)) => Availability::DoNotDisturb,
            (None, Some(presence::Show::XA)) => Availability::Away,
            (Some(_), _) => Availability::Unavailable,
        })
    }
}
