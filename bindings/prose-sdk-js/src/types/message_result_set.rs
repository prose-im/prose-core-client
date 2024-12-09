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
        self.0.messages.clone().into()
    }

    /// Can be used to load more messages. `lastMessageId` might not be contained in `messages`.
    /// If not set there are no more messages to load.
    #[wasm_bindgen(getter, js_name = "lastMessageId")]
    pub fn last_message_id(&self) -> Option<String> {
        self.0.last_message_id.clone().map(|id| id.to_string())
    }
}

impl From<CoreMessageResultSet> for MessageResultSet {
    fn from(value: CoreMessageResultSet) -> Self {
        Self(value)
    }
}
