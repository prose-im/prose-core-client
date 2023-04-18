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
