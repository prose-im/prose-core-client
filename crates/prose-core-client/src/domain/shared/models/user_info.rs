// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;

use crate::dtos::Availability;

#[derive(Debug, Clone, PartialEq)]
pub struct UserBasicInfo {
    pub jid: BareJid,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserPresenceInfo {
    pub jid: BareJid,
    pub name: String,
    pub availability: Availability,
}
