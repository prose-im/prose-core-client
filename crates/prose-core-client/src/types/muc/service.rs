// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;

#[derive(Clone, Debug)]
pub(crate) struct Service {
    pub jid: BareJid,
}
