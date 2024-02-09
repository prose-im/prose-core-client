// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::anyhow;
use js_sys::{Object, Reflect};
use tracing::error;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};

use prose_core_client::dtos;

use crate::types::{Attachment, AttachmentsArray, IntoJSArray};

#[wasm_bindgen]
pub struct SendMessageRequest {
    body: Option<String>,
    attachments: Vec<Attachment>,
}

#[wasm_bindgen]
impl SendMessageRequest {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            body: None,
            attachments: vec![],
        }
    }

    #[wasm_bindgen(getter)]
    pub fn body(&self) -> Option<String> {
        self.body.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_body(&mut self, body: Option<String>) {
        self.body = body;
    }

    #[wasm_bindgen(getter)]
    pub fn attachments(&self) -> AttachmentsArray {
        self.attachments.iter().cloned().collect_into_js_array()
    }

    #[wasm_bindgen(setter)]
    pub fn set_attachments(&mut self, attachments: AttachmentsArray) {
        let js_val: &JsValue = attachments.as_ref();
        let array: Option<&js_sys::Array> = js_val.dyn_ref();

        let Some(array) = array else {
            error!("Tried to assign a non-Array to 'attachments' of 'SendMessageRequest'.");
            return;
        };

        let Ok(length) = usize::try_from(array.length()) else {
            error!("Could not determine the length of the attachments array.");
            return;
        };

        let mut typed_array = Vec::<Attachment>::with_capacity(length);
        for js in array.iter() {
            let obj = match js.dyn_into::<js_sys::Object>() {
                Ok(obj) => obj,
                Err(err) => {
                    error!("Failed to parse attachment. {:?}", err);
                    return;
                }
            };

            let attachment = match Attachment::try_from(obj) {
                Ok(attachment) => attachment,
                Err(err) => {
                    error!("Failed to parse attachment. {}", err.to_string());
                    return;
                }
            };
            typed_array.push(attachment);
        }

        self.attachments = typed_array;
    }
}

impl TryFrom<SendMessageRequest> for dtos::SendMessageRequest {
    type Error = anyhow::Error;

    fn try_from(value: SendMessageRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            body: value.body,
            attachments: value
                .attachments
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<Attachment> for dtos::Attachment {
    type Error = anyhow::Error;

    fn try_from(value: Attachment) -> Result<Self, Self::Error> {
        Ok(Self {
            url: value.url.parse()?,
            description: value.description,
        })
    }
}

impl TryFrom<Object> for Attachment {
    type Error = anyhow::Error;

    fn try_from(value: Object) -> Result<Self, Self::Error> {
        let url = Reflect::get(&value, &JsValue::from_str("url"))
            .ok()
            .and_then(|value| value.as_string())
            .ok_or_else(|| anyhow!("url is not a String"))?;

        let description = Reflect::get(&value, &JsValue::from_str("description"))
            .ok()
            .and_then(|value| value.as_string());

        Ok(Attachment { url, description })
    }
}
