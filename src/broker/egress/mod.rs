// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Modules --

mod messaging;

// -- Imports --

use messaging::ProseBrokerEgressMessaging;

// -- Structures --

#[derive(Default)]
pub struct ProseBrokerEgress {
    messaging: ProseBrokerEgressMessaging,
}

// -- Implementations --

impl ProseBrokerEgress {
    pub fn new() -> Self {
        ProseBrokerEgress::default()
    }
}
