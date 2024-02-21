// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use wasm_bindgen::prelude::wasm_bindgen;

use prose_core_client::dtos::MessageResultSet as CoreMessageResultSet;

use crate::types::MessagesArray;

#[wasm_bindgen]
pub struct MessageResultSet(CoreMessageResultSet);

#[wasm_bindgen]
impl MessageResultSet {
    /// The requested messages in the order from oldest to newest.
    #[wasm_bindgen(getter)]
    pub fn messages(self) -> MessagesArray {
        self.0.messages.into()
    }

    /// Is `true` when the `MessageResultSet` contains messages from the last (earliest by date)
    /// page.
    #[wasm_bindgen(getter, js_name = "isLast")]
    pub fn is_last(&self) -> bool {
        self.0.is_last
    }
}

impl From<CoreMessageResultSet> for MessageResultSet {
    fn from(value: CoreMessageResultSet) -> Self {
        Self(value)
    }
}
