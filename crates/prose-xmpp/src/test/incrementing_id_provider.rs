// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;
use std::sync::{Arc, Mutex};

use crate::IDProvider;

pub struct IncrementingIDProvider {
    prefix: String,
    last_id: Mutex<i64>,
}

impl IncrementingIDProvider {
    pub fn new(prefix: &str) -> Self {
        IncrementingIDProvider {
            prefix: prefix.to_string(),
            last_id: Mutex::new(0),
        }
    }

    pub fn reset(&self) {
        let mut last_id = self.last_id.lock().unwrap();
        *last_id = 0;
    }

    pub fn last_id(&self) -> String {
        let last_id = self.last_id.lock().unwrap();
        format!("{}-{}", self.prefix, *last_id)
    }
}

impl IDProvider for IncrementingIDProvider {
    fn new_id(&self) -> String {
        let mut last_id = self.last_id.lock().unwrap();
        *last_id += 1;
        format!("{}-{}", self.prefix, *last_id)
    }
}

impl IDProvider for Arc<IncrementingIDProvider> {
    fn new_id(&self) -> String {
        self.deref().new_id()
    }
}
