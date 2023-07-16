use wasm_bindgen::prelude::*;

use crate::types::Jid;

#[wasm_bindgen]
pub struct Message(prose_domain::Message);

#[wasm_bindgen]
impl Message {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.0.id.to_string()
    }

    #[wasm_bindgen(getter, js_name = "stanzaID")]
    pub fn stanza_id(&self) -> Option<String> {
        self.0.stanza_id.as_ref().map(|id| id.to_string())
    }

    #[wasm_bindgen(getter, js_name = "from")]
    pub fn from_(&self) -> Jid {
        jid::Jid::Bare(self.0.from.clone()).into()
    }

    #[wasm_bindgen(getter)]
    pub fn body(&self) -> String {
        self.0.body.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> js_sys::Date {
        let timestamp_ms = self.0.timestamp.timestamp_millis() as f64;
        let js_date = js_sys::Date::new(&JsValue::from(timestamp_ms));
        js_date
    }
}

impl From<prose_domain::Message> for Message {
    fn from(value: prose_domain::Message) -> Self {
        Message(value)
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Message[]")]
    pub type MessagesArray;
}
