// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::dtos::{Availability, UserId, UserStatus};

#[derive(Debug, PartialEq, Clone)]
pub struct AccountInfo {
    pub id: UserId,
    pub name: String,
    pub availability: Availability,
    pub status: Option<UserStatus>,
}
