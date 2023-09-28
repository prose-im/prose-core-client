// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::IntoJSArray;
use wasm_bindgen::prelude::*;

use super::{BareJid, BareJidArray, ReactionsArray};

#[wasm_bindgen]
pub struct Message(prose_core_client::types::Message);

#[wasm_bindgen]
pub struct Reaction {
    #[wasm_bindgen(skip)]
    pub emoji: String,
    #[wasm_bindgen(skip)]
    pub from: Vec<BareJid>,
}

#[wasm_bindgen]
impl Message {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> Option<String> {
        self.0.id.as_ref().map(|id| id.to_string())
    }

    #[wasm_bindgen(setter)]
    pub fn set_id(&mut self, id: Option<String>) {
        self.0.id = id.clone().map(Into::into)
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
            .map(|r| Reaction::from(r.clone()))
            .collect_into_js_array::<ReactionsArray>()
    }
}

#[wasm_bindgen]
impl Reaction {
    #[wasm_bindgen(getter, js_name = "reaction")]
    pub fn emoji(&self) -> String {
        self.emoji.clone()
    }

    #[wasm_bindgen(getter, js_name = "authors")]
    pub fn from_(&self) -> BareJidArray {
        self.from
            .iter()
            .cloned()
            .collect_into_js_array::<BareJidArray>()
    }
}

impl From<prose_core_client::types::Message> for Message {
    fn from(value: prose_core_client::types::Message) -> Self {
        Message(value)
    }
}

impl From<prose_core_client::types::Reaction> for Reaction {
    fn from(value: prose_core_client::types::Reaction) -> Self {
        Reaction {
            emoji: value.emoji.into_inner(),
            from: value.from.into_iter().map(Into::into).collect(),
        }
    }
}
