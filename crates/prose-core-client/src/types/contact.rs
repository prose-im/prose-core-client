// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::Availability;
use crate::types::UserActivity;
use jid::BareJid;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub jid: BareJid,
    pub name: String,
    pub availability: Availability,
    pub activity: Option<UserActivity>,
    pub groups: Vec<String>,
}
