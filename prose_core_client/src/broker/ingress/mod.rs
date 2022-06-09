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

pub struct ProseBrokerIngress<'cl, 'cb, 'cx> {
    client: &'cl ProseBrokerClient<'cb, 'cx>,
}

pub enum ProseBrokerIngressEvent {
    Messaging(ProseBrokerIngressEventMessaging),
}

// -- Implementations --

impl<'cl, 'cb, 'cx> ProseBrokerIngress<'cl, 'cb, 'cx> {
    pub fn new(client: &'cl ProseBrokerClient<'cb, 'cx>) -> Self {
        Self { client }
    }

    pub async fn listen(&self) {
        // TODO: emit messages as they come
    }
}
