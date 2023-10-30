// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;

#[derive(Debug, PartialEq, Clone)]
pub struct Bookmark {
    pub name: String,
    pub room_jid: BareJid,
}
