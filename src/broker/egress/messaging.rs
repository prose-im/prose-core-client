// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use jid::BareJid;

// -- Imports --

use super::ProseBrokerClient;

// -- Structures --

pub struct ProseBrokerEgressMessaging<'cl, 'cb, 'cx> {
    client: &'cl ProseBrokerClient<'cb, 'cx>,
}

// -- Implementations --

impl<'cl, 'cb, 'cx> ProseBrokerEgressMessaging<'cl, 'cb, 'cx> {
    pub fn new(client: &'cl ProseBrokerClient<'cb, 'cx>) -> Self {
        Self { client }
    }

    pub fn send_message(&self, _to: BareJid, _body: &str) {
        // TODO
    }
}
