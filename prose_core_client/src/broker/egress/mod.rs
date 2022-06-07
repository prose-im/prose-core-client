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

pub struct ProseBrokerEgress<'cl, 'cb, 'cx> {
    client: &'cl ProseBrokerClient<'cb, 'cx>,

    pub messaging: ProseBrokerEgressMessaging<'cl, 'cb, 'cx>,
}

// -- Implementations --

impl<'cl, 'cb, 'cx> ProseBrokerEgress<'cl, 'cb, 'cx> {
    pub fn new(client: &'cl ProseBrokerClient<'cb, 'cx>) -> Self {
        Self {
            client,

            messaging: ProseBrokerEgressMessaging::new(client),
        }
    }
}
