// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Modules --

mod egress;
mod ingress;

// -- Imports --

use std::sync::{Arc, RwLock};

use tokio_xmpp::AsyncClient as XMPPClient;

use egress::ProseBrokerEgress;
use ingress::ProseBrokerIngress;

// -- Types --

pub type ProseBrokerClient = Arc<RwLock<XMPPClient>>;

// -- Structures --

pub struct ProseBroker {
    pub egress: ProseBrokerEgress,
    pub ingress: ProseBrokerIngress,
}

// -- Implementations --

impl ProseBroker {
    pub fn new(client: ProseBrokerClient) -> Self {
        ProseBroker {
            egress: ProseBrokerEgress::new(client.clone()),
            ingress: ProseBrokerIngress::new(client.clone()),
        }
    }
}
