// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Modules --

mod messaging;

// -- Imports --

use super::ProseBrokerClient;
use messaging::ProseBrokerIngressEventMessaging;

// -- Structures --

pub struct ProseBrokerIngress {
    client: ProseBrokerClient,
}

pub enum ProseBrokerIngressEvent {
    Messaging(ProseBrokerIngressEventMessaging),
}

// -- Implementations --

impl ProseBrokerIngress {
    pub fn new(client: ProseBrokerClient) -> Self {
        ProseBrokerIngress { client }
    }

    pub async fn listen(&self) {
        // TODO: emit messages as they come
    }
}
