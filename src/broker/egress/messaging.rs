// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use jid::BareJid;

// -- Structures --

#[derive(Default)]
pub struct ProseBrokerEgressMessaging;

// -- Implementations --

impl ProseBrokerEgressMessaging {
    pub fn send_message(&self, _to: BareJid, _body: &str) {
        // TODO
    }
}
