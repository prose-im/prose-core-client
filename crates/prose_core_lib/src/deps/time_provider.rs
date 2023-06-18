use std::ops::Deref;
use std::sync::Arc;
use std::time::SystemTime;

pub trait TimeProvider: Send + Sync {
    fn now(&self) -> SystemTime;
}

pub struct SystemTimeProvider {}

impl SystemTimeProvider {
    pub fn new() -> Self {
        SystemTimeProvider {}
    }
}

impl TimeProvider for SystemTimeProvider {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

impl TimeProvider for Arc<dyn TimeProvider> {
    fn now(&self) -> SystemTime {
        self.deref().now()
    }
}
