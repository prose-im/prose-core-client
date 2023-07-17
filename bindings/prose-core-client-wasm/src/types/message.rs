use wasm_bindgen::prelude::*;

use crate::types::BareJid;

#[wasm_bindgen]
pub struct Message(prose_domain::Message);

#[wasm_bindgen]
pub struct Reaction {
    #[wasm_bindgen(skip)]
    pub emoji: String,
    #[wasm_bindgen(skip)]
    pub from: Vec<BareJid>,
}

//if (!t.id || !t.type || !t.date || !t.content || !t.from)

#[wasm_bindgen]
impl Message {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.0.id.to_string()
    }

    #[wasm_bindgen(setter)]
    pub fn set_id(&mut self, id: &str) {
        self.0.id = id.to_string().into()
    }

    #[wasm_bindgen(getter, js_name = "archiveId")]
    pub fn stanza_id(&self) -> Option<String> {
        self.0.stanza_id.as_ref().map(|id| id.to_string())
    }

    #[wasm_bindgen(getter, js_name = "from")]
    pub fn from_(&self) -> String {
        self.0.from.to_string()
    }

    #[wasm_bindgen(getter, js_name = "content")]
    pub fn body(&self) -> String {
        if self.0.body.is_empty() {
            return "<empty message>".to_string();
        }
        self.0.body.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn date(&self) -> String {
        "2023-07-17T09:40:48.746Z".to_string()
    }
    #[wasm_bindgen(getter, js_name = "type")]
    pub fn _type(&self) -> String {
        "text".to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> String {
        self.body()
    }
    #[wasm_bindgen(getter)]
    pub fn metas(&self) -> Option<String> {
        None
    }

    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> js_sys::Date {
        let timestamp_ms = self.0.timestamp.timestamp_millis() as f64;
        let js_date = js_sys::Date::new(&JsValue::from(timestamp_ms));
        js_date
    }

    #[wasm_bindgen(getter)]
    pub fn reactions(&self) -> ReactionsArray {
        self.0
            .reactions
            .iter()
            .map(|r| {
                let r: Reaction = r.clone().into();
                JsValue::from(r)
            })
            .collect::<js_sys::Array>()
            .unchecked_into::<ReactionsArray>()
    }
}

#[wasm_bindgen]
impl Reaction {
    #[wasm_bindgen(getter, js_name = "reaction")]
    pub fn emoji(&self) -> String {
        self.emoji.clone()
    }

    #[wasm_bindgen(getter, js_name = "authors")]
    pub fn from(&self) -> BareJidArray {
        self.from
            .iter()
            .map(|b| JsValue::from(b.clone()))
            .collect::<js_sys::Array>()
            .unchecked_into::<BareJidArray>()
    }
}

impl From<prose_domain::Message> for Message {
    fn from(value: prose_domain::Message) -> Self {
        Message(value)
    }
}

impl From<prose_domain::Reaction> for Reaction {
    fn from(value: prose_domain::Reaction) -> Self {
        Reaction {
            emoji: value.emoji.0,
            from: value.from.into_iter().map(Into::into).collect(),
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Message[]")]
    pub type MessagesArray;

    #[wasm_bindgen(typescript_type = "Reaction[]")]
    pub type ReactionsArray;

    #[wasm_bindgen(typescript_type = "BareJid[]")]
    pub type BareJidArray;
}
