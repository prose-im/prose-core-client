// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::contacts::models::Group;
use crate::domain::shared::models::{Availability, UserId};
use crate::domain::user_info::models::UserStatus;

#[derive(Debug, PartialEq, Clone)]
pub struct Contact {
    pub id: UserId,
    pub name: String,
    pub availability: Availability,
    pub activity: Option<UserStatus>,
    pub group: Group,
}
