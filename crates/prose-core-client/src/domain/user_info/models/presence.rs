// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::{Availability, CapabilitiesId};
use crate::domain::user_info::models::JabberClient;
use crate::dtos::Avatar;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Presence {
    pub availability: Availability,
    pub avatar: Option<Avatar>,
    pub caps: Option<CapabilitiesId>,
    pub client: Option<JabberClient>,
    pub nickname: Option<String>,
    pub priority: i8,
    pub status: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_unavailable() {
        assert_eq!(
            Presence::default(),
            Presence {
                availability: Availability::Unavailable,
                avatar: None,
                caps: None,
                client: None,
                nickname: None,
                priority: 0,
                status: None,
            }
        )
    }
}
