// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;

/// A user who is currently typing in a Room.
#[derive(Debug, Clone, PartialEq)]
pub struct ComposingUser {
    pub name: String,
    pub jid: BareJid,
}
