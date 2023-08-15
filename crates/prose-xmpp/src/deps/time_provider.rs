// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, FixedOffset, Local};
use std::ops::Deref;
use std::sync::Arc;

pub trait TimeProvider: Send + Sync {
    fn now(&self) -> DateTime<FixedOffset>;
}

#[derive(Default)]
pub struct SystemTimeProvider {}

impl TimeProvider for SystemTimeProvider {
    fn now(&self) -> DateTime<FixedOffset> {
        Local::now().into()
    }
}

impl TimeProvider for Arc<dyn TimeProvider> {
    fn now(&self) -> DateTime<FixedOffset> {
        self.deref().now()
    }
}
