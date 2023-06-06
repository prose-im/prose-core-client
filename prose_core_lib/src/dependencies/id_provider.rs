use std::ops::Deref;
use std::sync::Arc;
use uuid::Uuid;

pub trait IDProvider: Send + Sync {
    fn new_id(&self) -> String;
}

pub struct UUIDProvider {}

impl UUIDProvider {
    pub fn new() -> Self {
        UUIDProvider {}
    }
}

impl IDProvider for UUIDProvider {
    fn new_id(&self) -> String {
        Uuid::new_v4().to_string()
    }
}

impl IDProvider for Arc<dyn IDProvider> {
    fn new_id(&self) -> String {
        self.deref().new_id()
    }
}
