// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use parking_lot::Mutex;

pub struct IncrementingOffsettingIDProvider {
    prefix: String,
    state: Mutex<State>,
}

impl IncrementingOffsettingIDProvider {
    pub fn new(prefix: &str) -> Self {
        IncrementingOffsettingIDProvider {
            prefix: prefix.to_string(),
            state: Default::default(),
        }
    }

    pub fn id_with_offset(&self, offset: i64) -> String {
        self.id_with_value(self.state.lock().id_with_offset(offset))
    }

    pub fn last_id_with_offset(&self, offset: i64) -> String {
        self.id_with_value(self.state.lock().last_id_with_offset(offset))
    }

    pub fn set_offset(&self, offset: i64) {
        self.state.lock().set_offset(offset)
    }

    pub fn apply_offset(&self) {
        self.state.lock().apply_offset()
    }

    pub fn next_id(&self) -> String {
        self.id_with_value(self.state.lock().next_id())
    }
}

#[derive(Default)]
struct State {
    last_id: i64,
    offset: i64,
}

impl State {
    fn id_with_offset(&self, offset: i64) -> i64 {
        self.last_id + offset
    }

    fn last_id_with_offset(&self, offset: i64) -> i64 {
        self.last_id + self.offset - offset + 1
    }

    fn set_offset(&mut self, offset: i64) {
        self.offset = offset
    }

    fn apply_offset(&mut self) {
        self.last_id += self.offset;
        self.offset = 0;
    }

    fn next_id(&self) -> i64 {
        self.last_id + self.offset + 1
    }
}

impl IncrementingOffsettingIDProvider {
    fn id_with_value(&self, value: i64) -> String {
        format!("{}-{}", self.prefix, value)
    }
}
