// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::Availability;
use crate::types::{roster, UserActivity, UserProfile};
use crate::util::{concatenate_names, StringExt};
use jid::BareJid;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub jid: BareJid,
    pub name: String,
    pub availability: Availability,
    pub activity: Option<UserActivity>,
    pub group: roster::Group,
}

impl
    From<(
        roster::Item,
        Option<UserProfile>,
        Option<Availability>,
        Option<UserActivity>,
    )> for Contact
{
    fn from(
        value: (
            roster::Item,
            Option<UserProfile>,
            Option<Availability>,
            Option<UserActivity>,
        ),
    ) -> Self {
        let user_profile = value.1.unwrap_or_default();

        let name = concatenate_names(&user_profile.first_name, &user_profile.last_name)
            .or(user_profile.nickname)
            .or(value.0.name)
            .or(value
                .0
                .jid
                .node()
                .map(|node| node.to_uppercase_first_letter()))
            .unwrap_or(value.0.jid.to_string().to_uppercase_first_letter());

        Contact {
            jid: value.0.jid,
            name,
            availability: value.2.unwrap_or(Availability::Unavailable),
            activity: value.3,
            group: value.0.group,
        }
    }
}
