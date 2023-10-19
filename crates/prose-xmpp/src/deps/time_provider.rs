// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Local, Utc};
use std::ops::Deref;
use std::sync::Arc;

pub trait TimeProvider: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Default)]
pub struct SystemTimeProvider {}

impl TimeProvider for SystemTimeProvider {
    fn now(&self) -> DateTime<Utc> {
        Local::now().into()
    }
}

impl TimeProvider for Arc<dyn TimeProvider> {
    fn now(&self) -> DateTime<Utc> {
        self.deref().now()
    }
}
