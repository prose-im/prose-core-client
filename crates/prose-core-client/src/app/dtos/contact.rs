// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;

use crate::domain::contacts::models::Group;
use crate::domain::shared::models::Availability;
use crate::domain::user_info::models::UserActivity;

#[derive(Debug, PartialEq, Clone)]
pub struct Contact {
    pub jid: BareJid,
    pub name: String,
    pub availability: Availability,
    pub activity: Option<UserActivity>,
    pub group: Group,
}
