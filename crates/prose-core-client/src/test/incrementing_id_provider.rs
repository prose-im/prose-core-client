// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Mutex;

use prose_xmpp::IDProvider;

pub struct IncrementingIDProvider {
    last_id: Mutex<i64>,
}

impl IncrementingIDProvider {
    pub fn new() -> Self {
        IncrementingIDProvider {
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
        format!("id-{}", *last_id)
    }
}
