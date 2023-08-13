// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Local};
use std::ops::Deref;
use std::sync::Arc;

pub trait TimeProvider: Send + Sync {
    fn now(&self) -> DateTime<Local>;
}

#[derive(Default)]
pub struct SystemTimeProvider {}

impl TimeProvider for SystemTimeProvider {
    fn now(&self) -> DateTime<Local> {
        Local::now()
    }
}

impl TimeProvider for Arc<dyn TimeProvider> {
    fn now(&self) -> DateTime<Local> {
        self.deref().now()
    }
}
