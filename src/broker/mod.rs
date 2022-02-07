// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Modules --

mod egress;
mod ingress;

// -- Imports --

use egress::ProseBrokerEgress;

// -- Structures --

pub struct ProseBroker {
    pub egress: ProseBrokerEgress,
}

// -- Implementations --

impl ProseBroker {
    pub fn new() -> Self {
        ProseBroker {
            egress: ProseBrokerEgress::new(),
        }
    }
}
