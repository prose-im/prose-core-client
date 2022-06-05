// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Modules --

mod egress;
mod ingress;

// -- Imports --

use libstrophe::Connection;

use egress::ProseBrokerEgress;
use ingress::ProseBrokerIngress;

// -- Structures --

pub struct ProseBroker<'cl, 'cb, 'cx> {
    pub egress: ProseBrokerEgress<'cl, 'cb, 'cx>,
    pub ingress: ProseBrokerIngress<'cl, 'cb, 'cx>,
}

pub struct ProseBrokerClient<'cb, 'cx> {
    connection: Connection<'cb, 'cx>,
}

// -- Implementations --

impl<'cl, 'cb, 'cx> ProseBroker<'cl, 'cb, 'cx> {
    pub fn from_connection(connection: Connection<'cb, 'cx>) -> Self {
        let client = ProseBrokerClient { connection };

        Self {
            egress: ProseBrokerEgress::new(&client),
            ingress: ProseBrokerIngress::new(&client),
        }
    }
}
