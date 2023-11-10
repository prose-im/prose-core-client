// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;

#[derive(Debug, Clone, PartialEq)]
pub struct PublicRoomInfo {
    pub jid: BareJid,
    pub name: Option<String>,
}
