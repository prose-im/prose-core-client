// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::{Availability, CapabilitiesId};
use crate::domain::user_info::models::{Avatar, JabberClient, UserStatus};

#[derive(Clone, PartialEq, Debug, Default)]
pub struct UserInfo {
    pub activity: Option<UserStatus>,
    pub availability: Availability,
    pub avatar: Option<Avatar>,
    pub caps: Option<CapabilitiesId>,
    pub client: Option<JabberClient>,
}
