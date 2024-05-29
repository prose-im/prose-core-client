// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use parking_lot::Mutex;

use crate::app::event_handlers::MessageEvent;
use crate::domain::messaging::repos::OfflineMessagesRepository as OfflineMessagesRepositoryTrait;

pub struct OfflineMessagesRepository {
    events: Mutex<Vec<MessageEvent>>,
}

impl OfflineMessagesRepository {
    pub fn new() -> Self {
        Self {
            events: Default::default(),
        }
    }
}

impl OfflineMessagesRepositoryTrait for OfflineMessagesRepository {
    fn push(&self, event: MessageEvent) {
        self.events.lock().push(event);
    }

    fn drain(&self) -> Vec<MessageEvent> {
        self.events.lock().drain(..).collect()
    }
}
