// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Mutex;

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
}

impl IDProvider for IncrementingIDProvider {
    fn new_id(&self) -> String {
        let mut last_id = self.last_id.lock().unwrap();
        *last_id += 1;
        format!("{}-{}", self.prefix, *last_id)
    }
}
