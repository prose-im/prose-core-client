use wasm_bindgen::prelude::*;

use prose_core_client::{ClientDelegate, ClientEvent};

#[wasm_bindgen]
extern "C" {
    pub type JSClientDelegate;

    #[wasm_bindgen(constructor)]
    fn new() -> JSClientDelegate;

    #[wasm_bindgen(method)]
    fn messages_appended(this: &JSClientDelegate, conversation: String, ids: Vec<JsValue>);
}

pub struct Delegate {
    js: JSClientDelegate,
}

impl Delegate {
    pub fn new(js: JSClientDelegate) -> Self {
        Delegate { js }
    }
}

impl ClientDelegate for Delegate {
    fn handle_event(&self, event: ClientEvent) {
        match event {
            ClientEvent::ComposingUsersChanged { .. } => {}
            ClientEvent::ConnectionStatusChanged { .. } => {}
            ClientEvent::ContactChanged { .. } => {}
            ClientEvent::MessagesAppended {
                conversation,
                message_ids,
            } => self.js.messages_appended(
                conversation.to_string(),
                message_ids
                    .into_iter()
                    .map(|id| JsValue::from(id.0))
                    .collect(),
            ),
            ClientEvent::MessagesUpdated { .. } => {}
            ClientEvent::MessagesDeleted { .. } => {}
        }
    }
}
