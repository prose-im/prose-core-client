// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::Availability;
use serde::{Deserialize, Serialize};

use crate::domain::user_info::models::{AvatarInfo, UserActivity};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct UserInfo {
    pub avatar: Option<AvatarInfo>,
    pub activity: Option<UserActivity>,
    #[serde(skip_serializing)]
    pub availability: Availability,
}
