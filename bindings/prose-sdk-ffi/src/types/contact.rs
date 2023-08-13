// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::JID;
use prose_core_client::types::{Availability, Contact as ProseContact, UserActivity};

#[derive(Debug, PartialEq, Clone)]
pub struct Contact {
    pub jid: JID,
    pub name: String,
    pub availability: Availability,
    pub activity: Option<UserActivity>,
    pub groups: Vec<String>,
}

impl From<ProseContact> for Contact {
    fn from(value: ProseContact) -> Self {
        Contact {
            jid: value.jid.into(),
            name: value.name,
            availability: value.availability,
            activity: value.activity,
            groups: value.groups,
        }
    }
}
