// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::presence;

use crate::app::dtos::Availability;
use crate::domain::user_info::models::Presence;

impl From<presence::Presence> for Presence {
    fn from(value: presence::Presence) -> Self {
        Presence {
            priority: value.priority,
            availability: Availability::from((
                if value.type_ == presence::Type::None {
                    None
                } else {
                    Some(value.type_)
                },
                value.show,
            )),
            status: value.statuses.first_key_value().map(|v| v.1.clone()),
        }
    }
}
