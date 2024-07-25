// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::dtos::MessageId;
use prose_xmpp::{IDProvider, UUIDProvider};

pub trait MessageIdProvider: Send + Sync {
    fn new_id(&self) -> MessageId;
}

pub struct WrappingMessageIdProvider<T: IDProvider> {
    id_provider: T,
}

impl<T: IDProvider> WrappingMessageIdProvider<T> {
    pub fn new(id_provider: T) -> Self {
        Self { id_provider }
    }
}

impl WrappingMessageIdProvider<UUIDProvider> {
    pub fn uuid() -> Self {
        Self {
            id_provider: UUIDProvider::new(),
        }
    }
}

#[cfg(feature = "test")]
impl WrappingMessageIdProvider<prose_xmpp::test::IncrementingIDProvider> {
    pub fn incrementing(prefix: &str) -> Self {
        Self {
            id_provider: prose_xmpp::test::IncrementingIDProvider::new(prefix),
        }
    }
}

impl<T: IDProvider> MessageIdProvider for WrappingMessageIdProvider<T> {
    fn new_id(&self) -> MessageId {
        MessageId::from(self.id_provider.new_id())
    }
}
