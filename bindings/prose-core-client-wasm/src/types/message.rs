use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "Message")]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message(prose_domain::Message);

#[wasm_bindgen(js_class = "Message")]
impl Message {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.0.id.to_string()
    }

    #[wasm_bindgen(getter, js_name = "stanzaID")]
    pub fn stanza_id(&self) -> Option<String> {
        self.0.stanza_id.as_ref().map(|id| id.to_string())
    }

    #[wasm_bindgen(getter)]
    pub fn from(&self) -> String {
        self.0.from.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn body(&self) -> String {
        self.0.body.clone()
    }
}

impl From<prose_domain::Message> for Message {
    fn from(value: prose_domain::Message) -> Self {
        Message(value)
    }
}
