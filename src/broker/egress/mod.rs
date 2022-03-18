// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Modules --

mod messaging;

// -- Imports --

use super::ProseBrokerClient;
use messaging::ProseBrokerEgressMessaging;

// -- Structures --

pub struct ProseBrokerEgress {
    client: ProseBrokerClient,

    pub messaging: ProseBrokerEgressMessaging,
}

// -- Implementations --

impl ProseBrokerEgress {
    pub fn new(client: ProseBrokerClient) -> Self {
        ProseBrokerEgress {
            client,

            messaging: ProseBrokerEgressMessaging::default(),
        }
    }
}
